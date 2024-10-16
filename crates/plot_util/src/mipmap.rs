use std::cell::RefCell;

/// Adapted from: <https://github.com/nchechulin/mipmap-1d/>
use num_traits::{FromPrimitive, Num, ToPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MipMap1D<T: Num + ToPrimitive + FromPrimitive> {
    data: Vec<Vec<T>>,
}

impl<T: Num + ToPrimitive + FromPrimitive + Copy> MipMap1D<T> {
    pub fn new(source: Vec<T>) -> Self {
        let mut data = vec![source.clone()];
        let mut current = source;

        while current.len() > 1 {
            let mipmap = Self::downsample(&current);
            current.clone_from(&mipmap);
            data.push(mipmap);
        }

        Self { data }
    }

    /// Returns the total number of downsampled levels.
    /// Equal to `ceil(log2(source.len())`
    pub fn num_levels(&self) -> usize {
        self.data.len()
    }

    /// Returns the data on given level.
    /// Level `0` returns the source data; the higher the level, the higher the compression (i.e. smaller vectors are returned).
    /// If the level is out of bounds, returns None
    pub fn get_level(&self, level: usize) -> Option<&Vec<T>> {
        if level >= self.num_levels() {
            return None;
        }

        Some(&self.data[level])
    }

    /// Downsamples a vector to `ceil(len / 2)` elements.
    /// Currently, downsampling is done by averaging the pair of elements
    fn downsample(source: &[T]) -> Vec<T> {
        source
            .chunks(2)
            .map(|pair| match pair.len() {
                1 => pair[0],
                2 => T::from_f64(
                    (pair[0] + pair[1])
                        .to_f64()
                        .expect("Value not representable as an f64")
                        / 2.0,
                )
                .expect("Value not representable as an f64"),
                _ => panic!("Unsound condition"),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MipMapStrategy {
    Linear,
    Min,
    Max,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct LevelLookupCached {
    pixel_width: usize,
    x_bounds: (usize, usize),
    result_span: (usize, usize),
    result_idx: usize,
}

impl LevelLookupCached {
    pub fn is_equal(&self, pixel_width: usize, x_bounds: (usize, usize)) -> bool {
        // Compare bounds first because pixel_width is much more stable
        self.x_bounds == x_bounds && self.pixel_width == pixel_width
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MipMap2D<T: Num + ToPrimitive + FromPrimitive + PartialOrd> {
    strategy: MipMapStrategy,
    data: Vec<Vec<[T; 2]>>,
    most_recent_lookup: RefCell<LevelLookupCached>,
}

impl<T: Num + ToPrimitive + FromPrimitive + Copy + PartialOrd> MipMap2D<T> {
    pub fn new(source: Vec<[T; 2]>, strategy: MipMapStrategy, min_elements: usize) -> Self {
        let mut data = vec![source.clone()];
        let mut current = source;

        while current.len() > min_elements {
            let mipmap = Self::downsample(&current, strategy);
            current.clone_from(&mipmap);
            data.push(mipmap);
        }

        Self {
            data,
            strategy,
            most_recent_lookup: RefCell::new(LevelLookupCached::default()),
        }
    }

    /// Create a [`MipMap2D`] but don't include the base level. Retrieving level 0 will then return an empty vec.
    ///
    /// Useful to avoid multiple redundant copies of the source if creating multiple [`MipMap2D`] from the same source.
    pub fn without_base(source: &[[T; 2]], strategy: MipMapStrategy, min_elements: usize) -> Self {
        // Include an empty vector at level 0 to make the levels align with what they normally would.
        let mut data = vec![Vec::<[T; 2]>::default()];

        let mut current = Self::downsample(source, strategy);
        if current.len() > min_elements {
            data.push(current.clone());
            while current.len() > min_elements {
                let mipmap = Self::downsample(&current, strategy);
                current.clone_from(&mipmap);
                data.push(mipmap);
            }
        }

        Self {
            data,
            strategy,
            most_recent_lookup: RefCell::new(LevelLookupCached::default()),
        }
    }

    /// Returns the total number of downsampled levels.
    /// Equal to `ceil(log2(source.len())`
    pub fn num_levels(&self) -> usize {
        self.data.len()
    }

    /// Returns the data on given level.
    /// Level `0` returns the source data; the higher the level, the higher the compression (i.e. smaller vectors are returned).
    /// If the level is out of bounds, returns None
    pub fn get_level(&self, level: usize) -> Option<&[[T; 2]]> {
        if level >= self.num_levels() {
            return None;
        }

        Some(self.data[level].as_slice())
    }

    /// Convenience function to get a level or return the highest if the requested level is higher or equal to the max
    pub fn get_level_or_max(&self, level: usize) -> &[[T; 2]] {
        if level >= self.num_levels() {
            return self.get_max_level();
        }

        &self.data[level]
    }

    /// Get the highest level of downsampling
    pub fn get_max_level(&self) -> &[[T; 2]] {
        &self.data[self.num_levels() - 1]
    }

    /// Downsamples a vector to `ceil(len / 2)` elements with the chosen [`MipMapStrategy`]
    fn downsample(source: &[[T; 2]], strategy: MipMapStrategy) -> Vec<[T; 2]> {
        let strategy = match strategy {
            MipMapStrategy::Linear => |point_pair: &[[T; 2]]| {
                let x = Self::linear_interpolate(point_pair[0][0], point_pair[1][0])
                    .expect("Interpolation error, this is a bug, please report it");
                let y = Self::linear_interpolate(point_pair[0][1], point_pair[1][1])
                    .expect("Interpolation error, this is a bug, please report it");
                [x, y]
            },
            MipMapStrategy::Min => |pairs: &[[T; 2]]| {
                // Branchless way of selecting the point with the smallest X-value
                let index_bool: usize = (pairs[0][1] > pairs[1][1]) as usize;
                pairs[index_bool]
            },
            MipMapStrategy::Max => |pairs: &[[T; 2]]| {
                // Branchless way of selecting the point with the greatest X-value
                let index_bool: usize = (pairs[0][1] < pairs[1][1]) as usize;
                pairs[index_bool]
            },
        };
        source
            .chunks(2)
            .map(|pairs| match pairs.len() {
                1 => pairs[0],
                2 => strategy(pairs),
                _ => unreachable!("Unsound condition"),
            })
            .collect()
    }

    fn linear_interpolate(a: T, b: T) -> Option<T> {
        let res = T::from_f64((a + b).to_f64()? / 2.0)?;
        Some(res)
    }

    /// Retrieves the index of the level that matches the specified pixel width
    /// and bounds within the dataset, using cached results if available.
    ///
    /// The bounds (`x_bounds`) are specified as a tuple `(x_min, x_max)`, which
    /// defines the range within which the data points must fall. The pixel width
    /// indicates how many data points should be considered in the calculation.
    ///
    /// # Parameters
    ///
    /// - `pixel_width`: The width in pixels that influences the number of points
    ///   considered for each level. This effectively determines the resolution of
    ///   the search.
    /// - `x_bounds`: A tuple containing two `usize` values `(x_min, x_max)` that
    ///   define the lower and upper bounds of the data points to be considered
    ///   for matching. These bounds are used to filter the relevant points in
    ///   each level.
    ///
    /// # Returns
    ///
    /// The level that matches the requirement (or the highest resolution), and if
    /// the match was found in a level below the max resolution, also returns a tuple
    /// of the start and end index of the level that matches the requirement.
    pub fn get_level_match(
        &self,
        pixel_width: usize,
        x_bounds: (usize, usize),
    ) -> (usize, Option<(usize, usize)>) {
        if self
            .most_recent_lookup
            .borrow()
            .is_equal(pixel_width, x_bounds)
        {
            return (
                self.most_recent_lookup.borrow().result_idx,
                Some(self.most_recent_lookup.borrow().result_span),
            );
        }
        let target_point_count = pixel_width;
        let (x_min, x_max) = x_bounds;

        // Avoid repeated calls to num_levels()
        let num_levels = self.num_levels();

        // If not found in cache, compute it
        for lvl_idx in (0..num_levels).rev() {
            let lvl = &self.data[lvl_idx];

            // Skip if the level doesn't have enough points even without accounting for plot bounds
            if lvl.len() <= target_point_count {
                continue;
            }

            let start_idx = lvl.partition_point(|x| fast_unix_ns_to_usize(x[0]) < x_min);
            let end_idx = lvl.partition_point(|x| fast_unix_ns_to_usize(x[0]) < x_max);

            // Use saturating_sub for safety and to avoid potential panic
            let count_within_bounds = end_idx.saturating_sub(start_idx);

            if count_within_bounds > target_point_count {
                let new_cached = LevelLookupCached {
                    pixel_width,
                    x_bounds,
                    result_span: (start_idx, end_idx),
                    result_idx: lvl_idx,
                };
                self.most_recent_lookup.replace(new_cached);
                return (lvl_idx, Some((start_idx, end_idx)));
            }
        }
        (0, None)
    }
}

/// Converts a unix timestamp in nanoseconds to `usize`.
///
/// This function is highly optimized for performance.
#[inline(always)]
#[allow(
    clippy::needless_pass_by_value,
    reason = "It's a numeric primitive, we don't mind passing by value"
)]
pub fn fast_unix_ns_to_usize<T: Num + ToPrimitive + FromPrimitive + PartialOrd>(
    unix_ts_ns: T,
) -> usize {
    // On 64-bit platforms, we can assume that `unix_ts_ns` fits in usize, so we just cast it directly.
    #[cfg(not(target_pointer_width = "32"))]
    #[allow(unsafe_code)]
    // SAFETY:
    // Assumes:
    // - That `usize` is at least 64 bits.
    // - That the argument is a unix timestamp that is less than the year ~2554
    unsafe {
        unix_ts_ns.to_usize().unwrap_unchecked()
    }
    #[cfg(target_pointer_width = "32")]
    unix_ts_ns.to_usize().expect("Doesn't fit in usize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[cfg(not(target_pointer_width = "32"))]
    const UNIX_TS_NS: usize = 1_728_470_564_000_000_000;
    #[cfg(target_pointer_width = "32")]
    const UNIX_TS_NS: usize = 1_728_470_564;

    #[test]
    fn test_mipmap_strategy_max() {
        let source: Vec<[f64; 2]> = vec![[1.1, 2.2], [3.3, 4.4], [5.5, 1.1], [7.7, 3.3]];
        let mipmap = MipMap2D::new(source, MipMapStrategy::Max, 1);

        // Level 1 should keep the points with higher y-values from each pair
        let expected_level_1: Vec<[f64; 2]> = vec![[3.3, 4.4], [7.7, 3.3]];
        assert_eq!(mipmap.get_level(1), Some(expected_level_1.as_slice()));

        // Level 2 should keep the point with the higher y-value from level 1
        let expected_level_2: Vec<[f64; 2]> = vec![[3.3, 4.4]];
        assert_eq!(mipmap.get_level(2), Some(expected_level_2.as_slice()));
    }

    #[test]
    fn test_mipmap_strategy_min() {
        let source: Vec<[f64; 2]> = vec![[1.1, 2.2], [3.3, 4.4], [5.5, 1.1], [7.7, 3.3]];
        let mipmap = MipMap2D::new(source, MipMapStrategy::Min, 1);

        // Level 1 should keep the points with lower y-values from each pair
        let expected_level_1: Vec<[f64; 2]> = vec![[1.1, 2.2], [5.5, 1.1]];
        assert_eq!(mipmap.get_level(1), Some(expected_level_1.as_slice()));

        // Level 2 should keep the point with the lower y-value from level 1
        let expected_level_2: Vec<[f64; 2]> = vec![[5.5, 1.1]];
        assert_eq!(mipmap.get_level(2), Some(expected_level_2.as_slice()));
    }

    #[test]
    fn test_mipmap_strategy_linear() {
        let source: Vec<[f64; 2]> = vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0], [7.0, 8.0]];
        let mipmap = MipMap2D::new(source.clone(), MipMapStrategy::Linear, 1);

        assert_eq!(mipmap.num_levels(), 3);
        assert_eq!(mipmap.get_level(0), Some(source.as_slice()));

        let expected_level_1: Vec<[f64; 2]> = vec![[2.0, 3.0], [6.0, 7.0]];
        assert_eq!(mipmap.get_level(1), Some(expected_level_1.as_slice()));

        let expected_level_2: Vec<[f64; 2]> = vec![[4.0, 5.0]];
        assert_eq!(mipmap.get_level(2), Some(expected_level_2.as_slice()));
    }

    #[test]
    fn test_different_strategies() {
        let source: Vec<[f64; 2]> = vec![[1.0, 2.0], [3.0, 8.0], [5.0, 4.0], [7.0, 10.0]];

        let linear_mipmap = MipMap2D::new(source.clone(), MipMapStrategy::Linear, 1);
        let min_mipmap = MipMap2D::new(source.clone(), MipMapStrategy::Min, 1);
        let max_mipmap = MipMap2D::new(source.clone(), MipMapStrategy::Max, 1);

        // Test level 1 for each strategy
        let expected_linear: Vec<[f64; 2]> = vec![[2.0, 5.0], [6.0, 7.0]];
        let expected_min: Vec<[f64; 2]> = vec![[1.0, 2.0], [5.0, 4.0]];
        let expected_max: Vec<[f64; 2]> = vec![[3.0, 8.0], [7.0, 10.0]];

        assert_eq!(linear_mipmap.get_level(1), Some(expected_linear.as_slice()));
        assert_eq!(min_mipmap.get_level(1), Some(expected_min.as_slice()));
        assert_eq!(max_mipmap.get_level(1), Some(expected_max.as_slice()));
    }

    #[test]
    fn test_get_level_or_max() {
        let source: Vec<[f64; 2]> = vec![[1.0, 2.0], [3.0, 4.0]];
        let mipmap = MipMap2D::new(source.clone(), MipMapStrategy::Linear, 1);

        assert_eq!(mipmap.get_level_or_max(0), &[[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(mipmap.get_level_or_max(1), &[[2.0, 3.0]]);
        assert_eq!(mipmap.get_level_or_max(2), &[[2.0, 3.0]]); // Returns max level
    }

    #[test]
    fn test_get_max_level() {
        let source: Vec<[f64; 2]> = vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0], [7.0, 8.0]];
        let mipmap = MipMap2D::new(source, MipMapStrategy::Linear, 1);

        assert_eq!(mipmap.get_max_level(), &[[4.0, 5.0]]);
    }

    #[test]
    fn test_level_match() {
        // Length of 16 will yield 5 levels of length:
        // - [0]: 16
        // - [1]: 8
        // - [2]: 4
        // - [3]: 2
        // - [4]: 1
        let source_len = 16;
        let source: Vec<[f64; 2]> = (0..source_len).map(|i| [i as f64, i as f64]).collect();
        let mipmap = MipMap2D::new(source, MipMapStrategy::Min, 1);

        for (pixel_width, expected_lvl, expected_range) in [
            (1usize, 3usize, Some((0, 2))),
            (2, 2, Some((0, 4))),
            (4, 1, Some((0, 8))),
            (8, 0, Some((0, 15))),
            (16, 0, None),
        ] {
            let (lvl, range_res) = mipmap.get_level_match(pixel_width, (0, 15));
            assert_eq!(
                lvl, expected_lvl,
                "Expected lvl {expected_lvl} for width: {pixel_width}"
            );
            assert_eq!(
                range_res, expected_range,
                "Expected range {expected_range:?} for width: {pixel_width}"
            );
        }
    }

    /// Test for: `<https://github.com/luftkode/logviewer-rs/issues/62>`
    #[test]
    fn test_level_match_large_timestamps() {
        let source_len = 1600;
        let source: Vec<[f64; 2]> = (0..source_len)
            .map(|i| [(i + UNIX_TS_NS) as f64, (i + UNIX_TS_NS) as f64])
            .collect();
        let mipmap = MipMap2D::new(source, MipMapStrategy::Min, 1);

        let x_bounds = (UNIX_TS_NS + 300, UNIX_TS_NS + 5000);

        for (pixel_width, expected_lvl, expected_range) in [
            (100usize, 3usize, Some((48, 200))),
            (200, 2, Some((96, 400))),
            (400, 1, Some((192, 800))),
            (800, 0, Some((384, 1600))),
            (1600, 0, None),
        ] {
            let (lvl, range_res) = mipmap.get_level_match(pixel_width, x_bounds);
            assert_eq!(
                lvl, expected_lvl,
                "Expected lvl {expected_lvl} for width: {pixel_width}"
            );

            assert_eq!(
                range_res, expected_range,
                "Expected range {expected_range:?} for width: {pixel_width}"
            );
        }
    }

    #[test]
    fn test_out_of_bounds_level() {
        let source: Vec<[f64; 2]> = vec![[1.0, 2.0], [3.0, 4.0]];
        let mipmap = MipMap2D::new(source, MipMapStrategy::Min, 1);

        assert_eq!(mipmap.get_level(2), None);
    }

    #[test]
    fn test_single_element_source() {
        let source: Vec<[f64; 2]> = vec![[1.0, 2.0]];
        let mipmap = MipMap2D::new(source.clone(), MipMapStrategy::Max, 1);

        assert_eq!(mipmap.num_levels(), 1);
        assert_eq!(mipmap.get_level(0), Some(source.as_slice()));
    }
}
