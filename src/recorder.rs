use raylib::texture::Image;

use crate::ringbuffer::RingBuffer;

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

    pub fn save_as_video(&self, filepath: &str) {
        let tmp_dir = std::env::temp_dir();
        // Concatenate a UUID to ensure there is no clash with other
        // pngs already in the tmp_dir
        let uuid = uuid::Uuid::new_v4();

        for (i, frame) in self.frames.into_iter().enumerate() {
            let mut frame_path = tmp_dir.clone();
            frame_path.push(format!("frame_{}_{}.png", uuid, i));
            frame.export_image(frame_path.to_str().unwrap());
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
