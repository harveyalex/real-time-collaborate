use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Encode a list of points into a compact byte representation.
/// Format: [count: u32 LE] [x0: f64 LE] [y0: f64 LE] [x1: f64 LE] [y1: f64 LE] ...
pub fn encode_points(points: &[Point]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + points.len() * 16);
    buf.extend_from_slice(&(points.len() as u32).to_le_bytes());
    for p in points {
        buf.extend_from_slice(&p.x.to_le_bytes());
        buf.extend_from_slice(&p.y.to_le_bytes());
    }
    buf
}

/// Decode points from the compact byte representation.
pub fn decode_points(data: &[u8]) -> Result<Vec<Point>, &'static str> {
    if data.len() < 4 {
        return Err("data too short for point count");
    }
    let count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let expected_len = 4 + count * 16;
    if data.len() < expected_len {
        return Err("data too short for declared point count");
    }
    let mut points = Vec::with_capacity(count);
    for i in 0..count {
        let offset = 4 + i * 16;
        let x = f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        let y = f64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap());
        points.push(Point { x, y });
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_empty() {
        let points: Vec<Point> = vec![];
        let encoded = encode_points(&points);
        let decoded = decode_points(&encoded).unwrap();
        assert_eq!(decoded, points);
    }

    #[test]
    fn encode_decode_multiple_points() {
        let points = vec![
            Point { x: 1.5, y: 2.5 },
            Point { x: -3.0, y: 100.0 },
            Point { x: 0.0, y: 0.0 },
        ];
        let encoded = encode_points(&points);
        let decoded = decode_points(&encoded).unwrap();
        assert_eq!(decoded, points);
    }

    #[test]
    fn decode_too_short_fails() {
        assert!(decode_points(&[]).is_err());
        assert!(decode_points(&[1, 0, 0, 0]).is_err()); // claims 1 point but no data
    }
}
