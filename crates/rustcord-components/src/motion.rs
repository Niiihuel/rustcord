//! How Discord's icons move under the pointer, as measured on its client
//! (`docs/discord-reference.md`, «Animated icons»): its Lottie animations
//! reimplemented as limn icon motions, not copied.

use limn::{IconKeyframe, IconMotion};

/// The settings gear: still for 120 ms, then half a turn clockwise, held.
pub fn gear() -> IconMotion {
    IconMotion::new()
        .keyframe(IconKeyframe::at(120))
        .keyframe(IconKeyframe::at(1070).turn(180.0))
}

/// A muted microphone rocks to one side, to the other, and settles.
pub fn muted_microphone() -> IconMotion {
    IconMotion::new()
        .keyframe(IconKeyframe::at(160).turn(-5.0))
        .keyframe(IconKeyframe::at(400).turn(5.0))
        .keyframe(IconKeyframe::at(650))
}

/// The headphones squash and settle.
pub fn headphones() -> IconMotion {
    IconMotion::new()
        .keyframe(IconKeyframe::at(150).scale(1.06, 0.88))
        .keyframe(IconKeyframe::at(350).scale(1.0, 1.04))
        .keyframe(IconKeyframe::at(600))
}
