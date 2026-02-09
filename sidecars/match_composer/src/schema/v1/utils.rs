pub fn pos_in_court(x: f32, y: f32) -> Result<(), &'static str> {
    if x < 0.0 || x > 1.0 {
        return Err("position x must be in [0, 1]");
    }

    if y < 0.0 || y > 1.0 {
        return Err("position y must be in [0, 1]");
    }
    
    Ok(())
}