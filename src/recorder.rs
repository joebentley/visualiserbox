use std::{io::Write, sync::mpsc};

use raylib::texture::{Image, ImageColors};

use crate::ringbuffer::RingBuffer;

struct ImageData {
    colors: ImageColors,
    width: u32,
    height: u32,
}

impl ImageData {
    fn to_rgb_u8_vec(&self) -> Vec<u8> {
        let mut rgb_u8_vec = Vec::new();
        for color in self.colors.iter() {
            rgb_u8_vec.push(color.r);
            rgb_u8_vec.push(color.g);
            rgb_u8_vec.push(color.b);
        }
        rgb_u8_vec
    }
}

pub struct ScreenRecorderState {
    steps_seen: usize,
    is_saving: bool,
    receiver: mpsc::Receiver<ScreenRecorderMessage>,
}

impl ScreenRecorderState {
    pub fn new(receiver: mpsc::Receiver<ScreenRecorderMessage>) -> Self {
        ScreenRecorderState {
            steps_seen: 0,
            is_saving: false,
            receiver,
        }
    }

    pub fn start(&mut self) {
        assert!(!self.is_saving);
        self.is_saving = true;
    }

    pub fn update(&mut self) {
        assert!(self.is_saving);
        if let Ok(value) = self.receiver.try_recv() {
            match value {
                ScreenRecorderMessage::ProcessingFrameStep => {
                    self.steps_seen += 1;
                }
                ScreenRecorderMessage::Done => {
                    self.reset();
                }
            }
        }
    }

    pub fn is_saving(&self) -> bool {
        self.is_saving
    }

    pub fn progress_string(&self, recorder_num_frames: usize) -> String {
        let percentage = 100 * self.steps_seen / recorder_num_frames;
        format!("{:3}% {}", percentage, "Rendering frames")
    }

    fn reset(&mut self) {
        assert!(self.is_saving);
        self.steps_seen = 0;
        self.is_saving = false;
    }
}

pub enum ScreenRecorderMessage {
    Done,
    ProcessingFrameStep,
}

pub struct ScreenRecorder {
    frames: RingBuffer<Image>,
    sender: mpsc::Sender<ScreenRecorderMessage>,
}

impl ScreenRecorder {
    pub fn new(length: usize, sender: mpsc::Sender<ScreenRecorderMessage>) -> ScreenRecorder {
        ScreenRecorder {
            frames: RingBuffer::new(length),
            sender,
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
        let image_datas = self.convert_frames_to_imagedata();

        let filepath = filepath.to_string();
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let width = image_datas[0].width;
            let height = image_datas[0].height;
            let mut bmp_frames = Vec::new();

            for image_data in image_datas {
                let mut rgb_frame_bytes = image_data.to_rgb_u8_vec();
                bmp_frames.append(&mut rgb_frame_bytes);
                sender
                    .send(ScreenRecorderMessage::ProcessingFrameStep)
                    .unwrap();
            }

            let mut child = ffmpeg_sidecar::command::FfmpegCommand::new()
                .args([
                    "-f",
                    "rawvideo",
                    "-pix_fmt",
                    "rgb24",
                    "-s",
                    format!("{}x{}", width, height).as_str(),
                    "-r",
                    "60",
                ])
                .input("-")
                .args(["-crf", "5"])
                .pix_fmt("yuv420p")
                .codec_video("libx264")
                .output(filepath)
                .print_command()
                .overwrite()
                .spawn()
                .unwrap();

            let mut stdin = child.take_stdin().unwrap();
            stdin.write_all(bmp_frames.as_slice()).unwrap();

            sender.send(ScreenRecorderMessage::Done).unwrap();
        });
    }
}
