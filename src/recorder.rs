use std::thread::JoinHandle;

use raylib::texture::{Image, ImageColors};

use crate::ringbuffer::RingBuffer;

struct ImageData {
    colors: ImageColors,
    width: u32,
    height: u32,
}

impl ImageData {
    fn to_rgbimage(&self) -> image::RgbImage {
        let mut rgb_image = image::RgbImage::new(self.width, self.height);
        for (i, color) in self.colors.iter().enumerate() {
            let x = i % (self.width as usize);
            let y = i / (self.width as usize);
            rgb_image.put_pixel(x as u32, y as u32, image::Rgb([color.r, color.g, color.b]));
        }
        rgb_image
    }
}

pub struct ScreenRecorder {
    frames: RingBuffer<Image>,
}

impl ScreenRecorder {
    pub fn new(length: usize) -> ScreenRecorder {
        ScreenRecorder {
            frames: RingBuffer::new(length),
        }
    }

    pub fn push_image(&mut self, image: Image) {
        self.frames.push(image)
    }

    fn convert_frames_to_imagedata(&self) -> Vec<ImageData> {
        let mut image_datas = Vec::new();

        for frame in self.frames.into_iter() {
            image_datas.push(ImageData {
                colors: frame.get_image_data(),
                width: frame.width as u32,
                height: frame.height as u32,
            });
        }

        image_datas
    }

    pub fn save_as_video(&self, filepath: &str) {
        let tmp_dir = std::env::temp_dir();
        // Concatenate a UUID to ensure there is no clash with other
        // pngs already in the tmp_dir
        let uuid = uuid::Uuid::new_v4();

        let mut handles: Vec<JoinHandle<_>> = Vec::new();
        let image_datas = self.convert_frames_to_imagedata();
        for (i, image_data) in image_datas.into_iter().enumerate() {
            let mut frame_path = tmp_dir.clone();
            frame_path.push(format!("frame_{}_{}.png", uuid, i));
            let handle = std::thread::spawn(move || {
                let frame = image_data.to_rgbimage();
                frame.save(frame_path.to_str().unwrap()).unwrap();
                println!("Saving frame {}", i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut frame_glob = tmp_dir.clone();
        frame_glob.push(format!("frame_{}_%d.png", uuid));

        let iter = ffmpeg_sidecar::command::FfmpegCommand::new()
            .input(frame_glob.to_str().unwrap())
            .args(["-pattern_type", "sequence"])
            .rate(30.0)
            .codec_video("libx264")
            .pix_fmt("yuv420p")
            .output(filepath)
            .print_command()
            .overwrite()
            .spawn()
            .unwrap()
            .iter()
            .unwrap();

        for message in iter.filter_errors() {
            eprintln!("{:#?}", message);
        }
    }
}
