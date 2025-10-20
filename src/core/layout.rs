use crate::core::types::LayoutMode;

/// Layout calculation result
#[derive(Debug, Clone, Copy)]
pub struct LayoutTransform {
    /// Source rectangle (x, y, width, height) in normalized coordinates [0, 1]
    pub src_rect: (f64, f64, f64, f64),

    /// Destination rectangle (x, y, width, height) in output pixels
    pub dst_rect: (i32, i32, i32, i32),
}

/// Calculate layout transform for rendering video to output
pub fn calculate_layout(
    mode: LayoutMode,
    video_width: i32,
    video_height: i32,
    output_width: i32,
    output_height: i32,
) -> LayoutTransform {
    let video_aspect = video_width as f64 / video_height as f64;
    let output_aspect = output_width as f64 / output_height as f64;

    match mode {
        LayoutMode::Fill | LayoutMode::Cover => {
            // Scale to cover entire output, crop video if needed
            if video_aspect > output_aspect {
                // Video is wider - crop sides
                let scale = output_height as f64 / video_height as f64;
                let scaled_width = video_width as f64 * scale;
                let crop_width = output_width as f64 / scaled_width;
                let crop_x = (1.0 - crop_width) / 2.0;

                LayoutTransform {
                    src_rect: (crop_x, 0.0, crop_width, 1.0),
                    dst_rect: (0, 0, output_width, output_height),
                }
            } else {
                // Video is taller - crop top/bottom
                let scale = output_width as f64 / video_width as f64;
                let scaled_height = video_height as f64 * scale;
                let crop_height = output_height as f64 / scaled_height;
                let crop_y = (1.0 - crop_height) / 2.0;

                LayoutTransform {
                    src_rect: (0.0, crop_y, 1.0, crop_height),
                    dst_rect: (0, 0, output_width, output_height),
                }
            }
        }

        LayoutMode::Contain => {
            // Scale to fit inside output, letterbox if needed
            if video_aspect > output_aspect {
                // Video is wider - letterbox top/bottom
                let scale = output_width as f64 / video_width as f64;
                let scaled_height = (video_height as f64 * scale) as i32;
                let offset_y = (output_height - scaled_height) / 2;

                LayoutTransform {
                    src_rect: (0.0, 0.0, 1.0, 1.0),
                    dst_rect: (0, offset_y, output_width, scaled_height),
                }
            } else {
                // Video is taller - letterbox left/right
                let scale = output_height as f64 / video_height as f64;
                let scaled_width = (video_width as f64 * scale) as i32;
                let offset_x = (output_width - scaled_width) / 2;

                LayoutTransform {
                    src_rect: (0.0, 0.0, 1.0, 1.0),
                    dst_rect: (offset_x, 0, scaled_width, output_height),
                }
            }
        }

        LayoutMode::Stretch => {
            // Stretch to fill, ignoring aspect ratio
            LayoutTransform {
                src_rect: (0.0, 0.0, 1.0, 1.0),
                dst_rect: (0, 0, output_width, output_height),
            }
        }

        LayoutMode::Centre => {
            // Center without scaling
            let offset_x = (output_width - video_width) / 2;
            let offset_y = (output_height - video_height) / 2;

            LayoutTransform {
                src_rect: (0.0, 0.0, 1.0, 1.0),
                dst_rect: (offset_x, offset_y, video_width, video_height),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_wider_video() {
        // 16:9 video on 4:3 display - should crop sides
        let transform = calculate_layout(LayoutMode::Fill, 1920, 1080, 1024, 768);
        assert_eq!(transform.dst_rect, (0, 0, 1024, 768));
        assert!(transform.src_rect.0 > 0.0); // Cropped from left
        assert!(transform.src_rect.2 < 1.0); // Cropped from right
    }

    #[test]
    fn test_contain_wider_video() {
        // 16:9 video on 4:3 display - should letterbox top/bottom
        let transform = calculate_layout(LayoutMode::Contain, 1920, 1080, 1024, 768);
        assert_eq!(transform.src_rect, (0.0, 0.0, 1.0, 1.0)); // Full source
        assert!(transform.dst_rect.1 > 0); // Offset from top
        assert!(transform.dst_rect.3 < 768); // Shorter than output
    }

    #[test]
    fn test_stretch() {
        let transform = calculate_layout(LayoutMode::Stretch, 1920, 1080, 1024, 768);
        assert_eq!(transform.src_rect, (0.0, 0.0, 1.0, 1.0));
        assert_eq!(transform.dst_rect, (0, 0, 1024, 768));
    }
}
