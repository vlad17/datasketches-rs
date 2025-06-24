use dsrs::{KllFloatSketch, KllDoubleSketch};

#[test]
fn test_kll_float_sketch_basic() {
    let mut sketch = KllFloatSketch::new();
    
    // Test that new sketch is empty
    assert!(sketch.is_empty());
    assert_eq!(sketch.get_n(), 0);
    assert_eq!(sketch.get_k(), 200); // Default k value
    
    // Add some values
    for i in 1..=100 {
        sketch.update(i as f32);
    }
    
    // Test that sketch is no longer empty
    assert!(!sketch.is_empty());
    assert_eq!(sketch.get_n(), 100);
    
    // Test quantile queries
    let median = sketch.get_quantile(0.5);
    assert!(median >= 40.0 && median <= 60.0); // Should be around 50
    
    let min_val = sketch.get_min_value();
    let max_val = sketch.get_max_value();
    assert_eq!(min_val, 1.0);
    assert_eq!(max_val, 100.0);
}

#[test]
fn test_kll_double_sketch_basic() {
    let mut sketch = KllDoubleSketch::new();
    
    // Test that new sketch is empty
    assert!(sketch.is_empty());
    assert_eq!(sketch.get_n(), 0);
    assert_eq!(sketch.get_k(), 200); // Default k value
    
    // Add some values
    for i in 1..=100 {
        sketch.update(i as f64);
    }
    
    // Test that sketch is no longer empty
    assert!(!sketch.is_empty());
    assert_eq!(sketch.get_n(), 100);
    
    // Test quantile queries
    let median = sketch.get_quantile(0.5);
    assert!(median >= 40.0 && median <= 60.0); // Should be around 50
    
    let min_val = sketch.get_min_value();
    let max_val = sketch.get_max_value();
    assert_eq!(min_val, 1.0);
    assert_eq!(max_val, 100.0);
}

#[test]
fn test_kll_float_sketch_with_k() {
    let sketch = KllFloatSketch::with_k(100);
    assert_eq!(sketch.get_k(), 100);
}

#[test]
fn test_kll_float_sketch_quantiles() {
    let mut sketch = KllFloatSketch::new();
    
    // Add values 1-100
    for i in 1..=100 {
        sketch.update(i as f32);
    }
    
    // Test multiple quantiles
    let fractions = [0.25, 0.5, 0.75];
    let quantiles = sketch.get_quantiles(&fractions);
    
    assert_eq!(quantiles.len(), 3);
    assert!(quantiles[0] >= 20.0 && quantiles[0] <= 30.0); // 25th percentile around 25
    assert!(quantiles[1] >= 45.0 && quantiles[1] <= 55.0); // 50th percentile around 50
    assert!(quantiles[2] >= 70.0 && quantiles[2] <= 80.0); // 75th percentile around 75
}

#[test]
fn test_kll_float_sketch_evenly_spaced() {
    let mut sketch = KllFloatSketch::new();
    
    // Add values 1-100
    for i in 1..=100 {
        sketch.update(i as f32);
    }
    
    // Test evenly spaced quantiles
    let quantiles = sketch.get_quantiles_evenly_spaced(5);
    assert_eq!(quantiles.len(), 5);
    
    // Should be approximately [1, 25, 50, 75, 100]
    assert_eq!(quantiles[0], 1.0); // min
    assert_eq!(quantiles[4], 100.0); // max
}

#[test]
fn test_kll_float_sketch_rank() {
    let mut sketch = KllFloatSketch::new();
    
    // Add values 1-100
    for i in 1..=100 {
        sketch.update(i as f32);
    }
    
    // Test rank queries
    let rank_of_50 = sketch.get_rank(50.0);
    assert!(rank_of_50 >= 0.45 && rank_of_50 <= 0.55); // Should be around 0.5
    
    let rank_of_1 = sketch.get_rank(1.0);
    assert!(rank_of_1 <= 0.05); // Should be near 0
    
    let rank_of_100 = sketch.get_rank(100.0);
    assert!(rank_of_100 >= 0.95); // Should be near 1
}

#[test]
fn test_kll_float_sketch_merge() {
    let mut sketch1 = KllFloatSketch::new();
    let mut sketch2 = KllFloatSketch::new();
    
    // Add different ranges to each sketch
    for i in 1..=50 {
        sketch1.update(i as f32);
    }
    
    for i in 51..=100 {
        sketch2.update(i as f32);
    }
    
    // Merge sketch2 into sketch1
    sketch1.merge(&sketch2);
    
    // Test that merged sketch has all values
    assert_eq!(sketch1.get_n(), 100);
    assert_eq!(sketch1.get_min_value(), 1.0);
    assert_eq!(sketch1.get_max_value(), 100.0);
}

#[test]
fn test_kll_float_sketch_serialization() {
    let mut sketch = KllFloatSketch::new();
    
    // Add some values
    for i in 1..=50 {
        sketch.update(i as f32);
    }
    
    // Serialize
    let serialized = sketch.serialize();
    let bytes = serialized.as_ref();
    assert!(!bytes.is_empty());
    
    // Deserialize
    let deserialized = KllFloatSketch::deserialize(bytes).expect("Failed to deserialize");
    
    // Test that deserialized sketch has same properties
    assert_eq!(deserialized.get_n(), 50);
    assert_eq!(deserialized.get_min_value(), 1.0);
    assert_eq!(deserialized.get_max_value(), 50.0);
    
    // Test quantile consistency
    let original_median = sketch.get_quantile(0.5);
    let deserialized_median = deserialized.get_quantile(0.5);
    assert!((original_median - deserialized_median).abs() < 1.0);
}

#[test]
fn test_kll_float_sketch_msgpack_serialization() {
    let mut sketch = KllFloatSketch::new();
    
    // Add some values
    for i in 1..=50 {
        sketch.update(i as f32);
    }
    
    // Serialize to MessagePack
    let msgpack_bytes = sketch.to_msgpack().expect("Failed to serialize to MessagePack");
    assert!(!msgpack_bytes.is_empty());
    
    // Deserialize from MessagePack
    let deserialized = KllFloatSketch::from_msgpack(&msgpack_bytes).expect("Failed to deserialize from MessagePack");
    
    // Test that deserialized sketch has same properties
    assert_eq!(deserialized.get_n(), 50);
    assert_eq!(deserialized.get_k(), sketch.get_k());
    assert_eq!(deserialized.get_min_value(), 1.0);
    assert_eq!(deserialized.get_max_value(), 50.0);
    
    // Test quantile consistency
    let original_median = sketch.get_quantile(0.5);
    let deserialized_median = deserialized.get_quantile(0.5);
    assert!((original_median - deserialized_median).abs() < 1.0);
}

#[test]
fn test_kll_double_sketch_msgpack_serialization() {
    let mut sketch = KllDoubleSketch::new();
    
    // Add some values
    for i in 1..=50 {
        sketch.update(i as f64);
    }
    
    // Serialize to MessagePack
    let msgpack_bytes = sketch.to_msgpack().expect("Failed to serialize to MessagePack");
    assert!(!msgpack_bytes.is_empty());
    
    // Deserialize from MessagePack
    let deserialized = KllDoubleSketch::from_msgpack(&msgpack_bytes).expect("Failed to deserialize from MessagePack");
    
    // Test that deserialized sketch has same properties
    assert_eq!(deserialized.get_n(), 50);
    assert_eq!(deserialized.get_k(), sketch.get_k());
    assert_eq!(deserialized.get_min_value(), 1.0);
    assert_eq!(deserialized.get_max_value(), 50.0);
    
    // Test quantile consistency
    let original_median = sketch.get_quantile(0.5);
    let deserialized_median = deserialized.get_quantile(0.5);
    assert!((original_median - deserialized_median).abs() < 1.0);
}