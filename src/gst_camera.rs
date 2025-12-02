use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GstCameraError {
    #[error("GStreamer initialization failed: {0}")]
    InitError(String),
    #[error("Pipeline creation failed: {0}")]
    PipelineError(String),
    #[error("Frame capture failed: {0}")]
    FrameError(String),
}

pub type Result<T> = std::result::Result<T, GstCameraError>;

pub struct GstCamera {
    pipeline: gst::Pipeline,
    appsink: gst_app::AppSink,
    width: u32,
    height: u32,
}

impl GstCamera {
    pub fn new(index: u32, width: u32, height: u32, fps: u32) -> Result<Self> {
        // Initialize GStreamer
        gst::init().map_err(|e| GstCameraError::InitError(e.to_string()))?;

        // Create pipeline for Bayer camera with conversion to RGB
        // The Imaging Source DFK 37BUX265 outputs RGGB Bayer format
        let pipeline_str = format!(
            "v4l2src device=/dev/video{} ! \
             video/x-bayer,format=rggb,width={},height={},framerate={}/1 ! \
             bayer2rgb ! \
             videoconvert ! \
             video/x-raw,format=RGB ! \
             appsink name=sink emit-signals=true sync=false max-buffers=1 drop=true",
            index, width, height, fps
        );

        eprintln!("Creating GStreamer pipeline: {}", pipeline_str);

        let pipeline = gst::parse::launch(&pipeline_str)
            .map_err(|e| GstCameraError::PipelineError(e.to_string()))?
            .dynamic_cast::<gst::Pipeline>()
            .map_err(|_| GstCameraError::PipelineError("Not a pipeline".to_string()))?;

        let appsink = pipeline
            .by_name("sink")
            .ok_or_else(|| GstCameraError::PipelineError("No appsink found".to_string()))?
            .dynamic_cast::<gst_app::AppSink>()
            .map_err(|_| GstCameraError::PipelineError("Not an appsink".to_string()))?;

        Ok(Self {
            pipeline,
            appsink,
            width,
            height,
        })
    }

    pub fn start(&self) -> Result<()> {
        eprintln!("Starting GStreamer pipeline...");
        self.pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| GstCameraError::PipelineError(format!("Failed to start: {}", e)))?;

        // Wait for pipeline to reach PLAYING state
        let _ = self.pipeline.state(gst::ClockTime::from_seconds(5));
        eprintln!("Pipeline started successfully");
        Ok(())
    }

    pub fn get_frame(&self) -> Result<Vec<u8>> {
        let sample = self
            .appsink
            .pull_sample()
            .map_err(|e| GstCameraError::FrameError(format!("Failed to pull sample: {}", e)))?;

        let buffer = sample
            .buffer()
            .ok_or_else(|| GstCameraError::FrameError("No buffer in sample".to_string()))?;

        let map = buffer
            .map_readable()
            .map_err(|e| GstCameraError::FrameError(format!("Failed to map buffer: {}", e)))?;

        let data = map.as_slice();
        let expected_size = (self.width * self.height * 3) as usize;

        if data.len() != expected_size {
            eprintln!(
                "Warning: Frame size mismatch. Expected {}, got {}",
                expected_size,
                data.len()
            );
        }

        Ok(data.to_vec())
    }

    pub fn stop(&self) -> Result<()> {
        self.pipeline
            .set_state(gst::State::Null)
            .map_err(|e| GstCameraError::PipelineError(format!("Failed to stop: {}", e)))?;
        Ok(())
    }

    pub fn get_resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for GstCamera {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Check if a camera uses Bayer format by querying v4l2
/// Returns true if the camera supports Bayer format (like The Imaging Source cameras)
pub fn is_bayer_camera(index: u32) -> bool {
    use std::process::Command;

    let device_path = format!("/dev/video{}", index);
    let output = match Command::new("v4l2-ctl")
        .arg("--device")
        .arg(&device_path)
        .arg("--list-formats")
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Failed to check if {} is Bayer camera: {}", device_path, e);
            return false;
        }
    };

    if output.status.success() {
        let info = String::from_utf8_lossy(&output.stdout);
        // Check if the output contains Bayer formats
        let is_bayer = info.contains("Bayer") || info.contains("RGGB") || info.contains("RG16") || info.contains("'RGGB'");
        if is_bayer {
            eprintln!("Detected Bayer format camera at {}", device_path);
        }
        is_bayer
    } else {
        false
    }
}
