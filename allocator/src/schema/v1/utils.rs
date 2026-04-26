use std::sync::LazyLock;

const COURT_X_RANGE: (f32, f32) = (-52.5, 52.5);
const COURT_Y_RANGE: (f32, f32) = (-34.0, 34.0);

static X_OUT_OF_COURT_MESSAGE: LazyLock<String> = LazyLock::new(|| format!("position x must be in [{}, {}]", COURT_X_RANGE.0, COURT_X_RANGE.1));
static Y_OUT_OF_COURT_MESSAGE: LazyLock<String> = LazyLock::new(|| format!("position y must be in [{}, {}]", COURT_Y_RANGE.0, COURT_Y_RANGE.1));


pub fn pos_in_court(x: f32, y: f32) -> Result<(), &'static str> {
    let (x_low, x_high) = COURT_X_RANGE;

    if x < x_low || x > x_high {
        return Err(&X_OUT_OF_COURT_MESSAGE);
    }

    let (y_low, y_high) = COURT_Y_RANGE;

    if y < y_low || y > y_high {
        return Err(&Y_OUT_OF_COURT_MESSAGE);
    }
    
    Ok(())
}