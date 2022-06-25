// OpenVR
extern crate openvr;
use openvr::{ApplicationType, TrackedControllerRole, Context};

pub struct RemiOVR {
    context: Option<Context>
}

impl RemiOVR {
    pub fn new() -> RemiOVR {
        let ctx: Option<Context> = match unsafe { openvr::init(ApplicationType::Background) } {
            Ok(ctx) => Some(ctx),
            _ => None
        };

        
        return RemiOVR { context: ctx };
    }

    pub fn has_context(&self) -> bool {
        return self.context.is_some();
    }

    pub fn vibrate_controller(&self, controller_role: TrackedControllerRole) {
        match self.context {
            Some(ref ctx) => {
                match ctx.system() {
                    Ok(sys) => {
                        println!("[*] Haptics triggered {:?}", controller_role);
                        let idx = sys.tracked_device_index_for_controller_role(controller_role).unwrap_or(0);
                        sys.trigger_haptic_pulse(idx, 1, 5000);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}