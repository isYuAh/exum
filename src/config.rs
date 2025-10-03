#[derive(Debug)]
pub struct ApplicationConfig {
  pub addr: [u8; 4],
  pub port: u16,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            addr: [0, 0, 0, 0],
            port: 8080,
        }
    }
}