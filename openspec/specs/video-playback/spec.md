# Video Playback

## Purpose
The video playback subsystem provides hardware-accelerated video decoding and rendering for desktop wallpapers, supporting multiple video formats, HDR content, and efficient resource management.

## Requirements

### Requirement: Hardware-Accelerated Decoding
The system SHALL support hardware-accelerated video decoding via VA-API and NVDEC.

#### Scenario: VA-API decoding on Intel/AMD GPUs
- **WHEN** a video file is loaded on a system with VA-API support
- **THEN** libmpv SHALL use VA-API hardware decoding
- **AND** CPU usage SHALL be significantly lower than software decoding

#### Scenario: NVDEC decoding on NVIDIA GPUs
- **WHEN** a video file is loaded on a system with NVIDIA GPU
- **THEN** libmpv SHALL use NVDEC hardware decoding
- **AND** decoding performance SHALL match or exceed VA-API

#### Scenario: Fallback to software decoding
- **WHEN** hardware decoding is unavailable or fails
- **THEN** libmpv SHALL fall back to software decoding
- **AND** a warning SHALL be logged about degraded performance

### Requirement: Multiple Video Format Support
The system SHALL support common video formats including MP4, WebM, MKV, AVI, and MOV.

#### Scenario: H.264/H.265 video playback
- **WHEN** a video file with H.264 or H.265 codec is loaded
- **THEN** the video SHALL play smoothly at native framerate
- **AND** no codec errors SHALL occur

#### Scenario: VP8/VP9 video playback
- **WHEN** a WebM video with VP8 or VP9 codec is loaded
- **THEN** the video SHALL decode and render correctly
- **AND** hardware acceleration SHALL be used if available

### Requirement: HDR Content Support
The system SHALL support 10-bit HDR video with automatic tone-mapping for SDR displays.

#### Scenario: HDR video on HDR display
- **WHEN** a 10-bit HDR video is played on an HDR-capable display
- **THEN** the video SHALL be rendered with full HDR color range
- **AND** no color clipping SHALL occur

#### Scenario: HDR to SDR tone-mapping
- **WHEN** a 10-bit HDR video is played on an SDR display
- **THEN** the video SHALL be tone-mapped to SDR color space
- **AND** the result SHALL preserve visual details without excessive brightness

#### Scenario: Tone-mapping algorithm selection
- **WHEN** HDR to SDR conversion is required
- **THEN** the system SHALL support multiple tone-mapping algorithms (ACES, Hable, Reinhard)
- **AND** the algorithm SHALL be configurable via config file

### Requirement: Frame Timing and Synchronization
The system SHALL maintain accurate frame timing synchronized with display refresh rate.

#### Scenario: VSync synchronization
- **WHEN** video is playing on a 60Hz display
- **THEN** frames SHALL be presented at 60 FPS without tearing
- **AND** frame timing SHALL align with display vsync

#### Scenario: Variable refresh rate support
- **WHEN** display supports variable refresh rate (Freesync/G-Sync)
- **THEN** frame timing SHALL adapt to video framerate
- **AND** playback SHALL be smooth without judder

#### Scenario: Frame drop handling
- **WHEN** decoding cannot keep up with playback rate
- **THEN** the system SHALL drop frames to maintain sync
- **AND** dropped frames SHALL be logged for diagnostics

### Requirement: Shared Decoder Optimization
The system SHALL share decoded video frames across multiple outputs playing the same source.

#### Scenario: Same video on multiple monitors
- **WHEN** two monitors display the same video file
- **THEN** video SHALL be decoded only once
- **AND** decoded frames SHALL be shared between outputs
- **AND** memory usage SHALL be approximately halved compared to independent decoding

#### Scenario: Different videos on multiple monitors
- **WHEN** monitors display different video files
- **THEN** each video SHALL have its own decoder instance
- **AND** decoders SHALL operate independently

### Requirement: Video Loop Behavior
The system SHALL seamlessly loop video playback without interruption.

#### Scenario: Continuous video loop
- **WHEN** a video reaches its end
- **THEN** playback SHALL immediately restart from the beginning
- **AND** no black screen or pause SHALL occur during loop transition

#### Scenario: Gapless loop with pre-buffering
- **WHEN** video is near the end
- **THEN** the next loop iteration SHALL be pre-buffered
- **AND** loop transition SHALL appear instantaneous

### Requirement: Resource Cleanup
The system SHALL properly release video resources when no longer needed.

#### Scenario: Video source change
- **WHEN** a new video source is loaded
- **THEN** the previous video decoder SHALL be stopped
- **AND** GPU textures and buffers SHALL be freed
- **AND** no memory leaks SHALL occur

#### Scenario: Graceful shutdown
- **WHEN** the daemon is terminated
- **THEN** all video decoders SHALL be stopped cleanly
- **AND** libmpv context SHALL be destroyed
- **AND** no zombie processes SHALL remain
