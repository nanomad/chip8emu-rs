use std::collections::HashSet;

pub struct Debugger {
	breakpoints: HashSet<usize>,
	paused: bool,
	step_count: usize
}

impl Debugger {
    pub fn new() -> Self {
    	Debugger {
    		breakpoints: HashSet::new(),
    		paused: false,
    		step_count: 0
    	}
    }

    pub fn add_breakpoint(&mut self, loc: usize) {
    	self.breakpoints.insert(loc);
    }

    pub fn current_location(&mut self, loc: &usize) {
    	self.step_count -= 1;
    	if self.must_break(loc) {
    		self.paused = true
    	}
    }

    pub fn is_paused(&self) -> bool {
    	self.paused
    }

    pub fn reset(&mut self) {
    	self.paused = false;
    }

    pub fn step(&mut self, count: usize) {
    	self.step_count = count;
    }

    fn must_break(&self, loc: &usize) -> bool {
    	(self.step_count <= 0) 
    	||
    	(!self.breakpoints.is_empty() && self.breakpoints.contains(loc))
    }
}