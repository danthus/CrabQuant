use std::collections::VecDeque;
pub struct MovingWindow {
    pub vector: VecDeque<f32>
}
impl MovingWindow {
    pub fn new(window_size: usize) -> Self {
        MovingWindow {
            vector: VecDeque::with_capacity(window_size)
        }
    }

    pub fn update(&mut self, new_value: f32) {
        if self.vector.len() == self.vector.capacity() {
            self.vector.pop_front();
        }
        self.vector.push_back(new_value);
    }

    pub fn average(&self, window_size: usize) -> f32 {
        if window_size <= self.vector.len() {
            self.vector.iter().skip(self.vector.len() - window_size).sum::<f32>() / (window_size as f32)
        }
        else {
            // if window size is larger, average entire vector
            self.vector.iter().sum::<f32>() / (self.vector.len() as f32)
        }
    }

    pub fn std(&self, window_size: usize) -> f32 {
        let avg = self.average(window_size);

        if window_size <= self.vector.len() {
            self.vector.iter().skip(self.vector.len() - window_size).map(|&x| (x - avg).powi(2)).sum::<f32>() / (window_size as f32)
        }
        else {
            self.vector.iter().map(|&x| (x - avg).powi(2)).sum::<f32>() / (self.vector.len() as f32)
        }   
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    fn init_vector() -> MovingWindow {
        mv = MovingWindow(10);
    }

    #[test]
    fn test_avg() {

    }
}