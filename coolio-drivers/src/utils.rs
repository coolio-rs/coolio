/// The normalization proces enshures that:
/// - the profile is a monotonically increasing function
///   (i.e. for every i, i > 1, x[i] - x[i-1] > 0 and y[i] - y[i-1] >= 0)
/// - the profile is sorted
/// - a (critx, 100) failsafe is enforced
/// - only the first point that sets y := 100 is kept
///
/// So duty value always always increase and last profile point ensures that for critical temp
/// we utilize max duty of 100%. Curve should look like ilustration below:
///
/// ```text
/// (duty) ^                  
///   100  │                   ┌─────────
///        │     ┌─────────────┘
///        │     /     
///    50  │ ___/
///        │
///        │
///        │
///     0  └────────────────────────────── >
///        20         40         60   (temp)
/// ```
pub fn normalize_profile(profile: Vec<(u8, u8)>, critical_temp: u8, min_duty: u8) -> Vec<(u8, u8)> {
  // ensure that duty is in the range min_duty..=100u8
  let sorted: &mut Vec<(u8, u8)> = &mut profile
    .iter()
    .map(|&(a, b)| (a, b.min(100u8).max(min_duty)))
    .collect();
  // sort
  sorted.sort_by(|&(t1, d1): &(u8, u8), &(t2, d2)| (t1, -(d1 as i16)).cmp(&(t2, -(d2 as i16))));
  // apend failsafe duty
  sorted.push((critical_temp, 100u8));
  // skip first (will be added in normalized vec)
  let rs: Vec<(u8, u8)> = sorted.iter().skip(1).map(|p| *p).collect();
  // skip last (it is included in above)
  let ls: Vec<(u8, u8)> = sorted.iter().rev().skip(1).rev().map(|p| *p).collect();
  // append first
  let normalized: &mut Vec<(u8, u8)> = &mut vec![*sorted.first().unwrap()];
  // zip ls and rs and do curve normalization
  for ((t1, d1), (t2, d2)) in rs.iter().zip(ls.iter()) {
    if t1 <= t2 {
      // must be higher temp, since previous is included
      continue;
    } else if d1 < d2 {
      // if next has lower duty we need to flaten that to match previous duty,
      // otherwise it may case cooliant overheat right!
      normalized.push((*t1, *d2))
    } else if *t1 >= critical_temp && *d1 >= 100 {
      // this implicitly means that we will include 100% duty if temp is not eqal or greater th
      break;
    } else {
      // this is ok, temp is higher and below critical cooliant temperature. Duty is also higher :)
      normalized.push((*t1, *d1))
    }
  }
  if normalized.last() < Some(&(critical_temp, 100u8)) {
    // include failsafe value in profile
    normalized.push((critical_temp, 100));
  }
  normalized.to_vec()
}

pub fn interpolate_profile(profile: &Vec<(u8, u8)>, temp: u8, critical_temp: u8) -> u8 {
  let default = (critical_temp as f32, 100f32);
  let profile_f32 = profile
    .iter()
    .map(|&(t, d)| (t as f32, d as f32))
    .collect::<Vec<_>>();
  let first = profile_f32.first().unwrap_or(&default);
  let &(upper_temp, upper_duty) = profile_f32
    .iter()
    .find(|&&(t, _d)| t >= (temp as f32))
    .unwrap_or(&default);
  let &(lower_temp, lower_duty) = profile_f32
    .iter()
    .rev()
    .find(|&&(t, _d)| t <= (temp as f32))
    .unwrap_or(first);

  if upper_temp == lower_temp {
    lower_duty as u8
  } else {
    let result = lower_duty
      + ((temp as f32) - lower_temp) / (upper_temp - lower_temp) * (upper_duty - lower_duty);
    result.round() as u8
  }
}
