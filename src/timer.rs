use std::cell::Cell;
use cgmath::num_traits::ToPrimitive;
use glfw::ffi::glfwGetTime;

pub struct DeltaTimer {
    begin: Cell<f64>,
    previous: Cell<f64>,
    dt: Cell<f64>,
    elapsed: Cell<f64>
}

impl DeltaTimer {
    pub fn new() -> Self {
        unsafe {
            let current = glfwGetTime().to_f64().unwrap();
            DeltaTimer {
                begin: Cell::new(current),
                previous: Cell::new(current),
                dt: Cell::new(0.0),
                elapsed: Cell::new(0.0)
            }
        }
    }

    pub fn update(&self) {
        let current_time = unsafe { glfwGetTime() }.to_f64().unwrap();
        // dt
        self.dt.set( current_time - self.previous.get());
        self.previous.set(current_time);
        // elapsed
        self.elapsed.set( current_time - self.begin.get());
    }

    pub fn dt(&self) -> f64 {
        self.dt.get()
    }

    pub fn elapsed(&self) -> f64 {
        self.elapsed.get()
    }

}