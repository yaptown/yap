//! A learner-local calendar of review activity, shared by both hosts.
use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::DaySummary;

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityDayView {
    pub date: String,
    pub reviews: u32,
    /// Zero is empty; 1–4 are quartiles of nonzero days in this calendar.
    pub level: u8,
    pub label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityWeekView {
    pub month_label: Option<String>,
    /// Monday first. Future days have no cell.
    pub days: Vec<Option<ActivityDayView>>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityView {
    pub title: String,
    pub summary: String,
    pub weekdays: Vec<String>,
    pub weeks: Vec<ActivityWeekView>,
    pub less_label: String,
    pub more_label: String,
}

pub(crate) fn activity_view(history: &BTreeMap<i64, DaySummary>, today: NaiveDate) -> ActivityView {
    let start = today - Duration::days(today.weekday().num_days_from_monday() as i64 + 52 * 7);
    let counts: Vec<u32> = (0..=(today - start).num_days())
        .map(|offset| {
            history
                .get(&i64::from(
                    (start + Duration::days(offset)).num_days_from_ce(),
                ))
                .map_or(0, |day| day.reviews)
        })
        .collect();
    let mut nonzero: Vec<_> = counts.iter().copied().filter(|&count| count > 0).collect();
    nonzero.sort_unstable();
    // Nearest-rank quartiles keep equal counts at the same intensity.
    let thresholds = [1, 2, 3].map(|q| {
        nonzero
            .get((nonzero.len() * q).div_ceil(4).saturating_sub(1))
            .copied()
            .unwrap_or(0)
    });
    let weeks = (0..53)
        .map(|week| {
            let monday = start + Duration::days(week * 7);
            let month_day = (0..7)
                .map(|day| monday + Duration::days(day))
                .find(|date| date.day() == 1 && *date <= today);
            ActivityWeekView {
                month_label: month_day.map(|date| month_label(date.month())),
                days: (0..7)
                    .map(|day| {
                        let date = monday + Duration::days(day);
                        if date > today {
                            return None;
                        }
                        let reviews = counts[(week * 7 + day) as usize];
                        let level = if reviews == 0 {
                            0
                        } else {
                            1 + thresholds
                                .iter()
                                .filter(|&&threshold| reviews > threshold)
                                .count() as u8
                        };
                        Some(ActivityDayView {
                            date: date.to_string(),
                            reviews,
                            level,
                            label: yap_i18n::tr!(
                                "activity-day",
                                count = reviews,
                                number = yap_i18n::format_count(reviews),
                                date = yap_i18n::tr!(
                                    "activity-date",
                                    year = date.year().to_string(),
                                    month = month_label(date.month()),
                                    day = date.day()
                                )
                            ),
                        })
                    })
                    .collect(),
            }
        })
        .collect();
    let total: u32 = counts.iter().sum();
    ActivityView {
        title: yap_i18n::tr!("activity-title"),
        summary: yap_i18n::tr!(
            "activity-summary",
            reviews = yap_i18n::tr!(
                "activity-review-count",
                count = total,
                number = yap_i18n::format_count(total)
            ),
            days = yap_i18n::tr!(
                "activity-day-count",
                count = nonzero.len(),
                number = yap_i18n::format_count(nonzero.len() as u32)
            )
        ),
        weekdays: vec![
            yap_i18n::tr!("activity-monday"),
            String::new(),
            yap_i18n::tr!("activity-wednesday"),
            String::new(),
            yap_i18n::tr!("activity-friday"),
            String::new(),
            String::new(),
        ],
        weeks,
        less_label: yap_i18n::tr!("activity-less"),
        more_label: yap_i18n::tr!("activity-more"),
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewTotalPoint {
    /// Days since the day before the first study day.
    pub day: u32,
    pub total: u32,
    pub label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewTotalTick {
    pub day: u32,
    pub label: String,
}

/// The running review total: zero the day before the first study day, then
/// one point per study day and one for today.
/// Hosts draw it as a step line, since the total only moves on study days.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewTotalView {
    pub title: String,
    pub points: Vec<ReviewTotalPoint>,
    pub ticks: Vec<ReviewTotalTick>,
}

pub(crate) fn review_total_view(
    history: &BTreeMap<i64, DaySummary>,
    today: NaiveDate,
) -> Option<ReviewTotalView> {
    let mut days = history
        .iter()
        .filter(|(_, day)| day.reviews > 0)
        .map(|(&day, summary)| {
            (
                NaiveDate::from_num_days_from_ce_opt(day as i32).unwrap(),
                summary.reviews,
            )
        })
        .peekable();
    // Starting from zero the day before keeps a first-day learner's chart a
    // line rather than a lone point.
    let origin = days.peek()?.0 - Duration::days(1);
    let mut last = origin;
    let mut total = 0;
    let mut points = vec![review_total_point(origin, origin, 0)];
    for (date, reviews) in days {
        total += reviews;
        last = date;
        points.push(review_total_point(origin, date, total));
    }
    // A device clock running fast can leave study days after today.
    if today > last {
        points.push(review_total_point(origin, today, total));
    }
    let span = points.last().unwrap().day;
    let intervals = span.min(4);
    let ticks = (0..=intervals)
        .map(|tick| {
            let day = span * tick / intervals;
            let date = origin + Duration::days(day.into());
            ReviewTotalTick {
                day,
                label: yap_i18n::tr!(
                    "activity-short-date",
                    month = month_label(date.month()),
                    day = date.day()
                ),
            }
        })
        .collect();
    Some(ReviewTotalView {
        title: yap_i18n::tr!("review-total-title"),
        points,
        ticks,
    })
}

fn review_total_point(first: NaiveDate, date: NaiveDate, total: u32) -> ReviewTotalPoint {
    ReviewTotalPoint {
        day: (date - first).num_days() as u32,
        total,
        label: yap_i18n::tr!(
            "review-total-point",
            date = yap_i18n::tr!(
                "activity-date",
                year = date.year().to_string(),
                month = month_label(date.month()),
                day = date.day()
            ),
            reviews = yap_i18n::tr!(
                "activity-review-count",
                count = total,
                number = yap_i18n::format_count(total)
            )
        ),
    }
}

fn month_label(month: u32) -> String {
    match month {
        1 => yap_i18n::tr!("activity-jan"),
        2 => yap_i18n::tr!("activity-feb"),
        3 => yap_i18n::tr!("activity-mar"),
        4 => yap_i18n::tr!("activity-apr"),
        5 => yap_i18n::tr!("activity-may"),
        6 => yap_i18n::tr!("activity-jun"),
        7 => yap_i18n::tr!("activity-jul"),
        8 => yap_i18n::tr!("activity-aug"),
        9 => yap_i18n::tr!("activity-sep"),
        10 => yap_i18n::tr!("activity-oct"),
        11 => yap_i18n::tr!("activity-nov"),
        12 => yap_i18n::tr!("activity-dec"),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_layout_and_quartiles() {
        let today = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let history = (0..8)
            .map(|offset| {
                (
                    i64::from((today - Duration::days(offset)).num_days_from_ce()),
                    DaySummary {
                        reviews: offset as u32 + 1,
                        time_spent_seconds: 0,
                        new_cards: 0,
                        learned_cards: 0,
                        locked_in_cards: 0,
                    },
                )
            })
            .collect();
        let view = activity_view(&history, today);
        assert_eq!(view.weeks.len(), 53);
        assert!(view.weeks.iter().all(|week| week.days.len() == 7));
        let days: Vec<_> = view
            .weeks
            .iter()
            .flat_map(|week| &week.days)
            .flatten()
            .collect();
        assert_eq!(days[0].date, "2023-02-27");
        assert_eq!(days.last().unwrap().date, "2024-03-01");
        assert_eq!(view.weeks[52].days[3].as_ref().unwrap().date, "2024-02-29");
        assert!(view.weeks[52].days[5..].iter().all(Option::is_none));
        assert!(view.weeks[52].month_label.is_some());
        for day in days {
            assert_eq!(day.level, day.reviews.div_ceil(2) as u8);
        }
        let empty = activity_view(&BTreeMap::new(), today);
        assert!(
            empty
                .weeks
                .iter()
                .flat_map(|week| &week.days)
                .flatten()
                .all(|day| day.level == 0)
        );
    }

    #[test]
    fn review_total_first_day_and_fast_clock() {
        let today = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let history = |date: NaiveDate| {
            BTreeMap::from([(
                i64::from(date.num_days_from_ce()),
                DaySummary {
                    reviews: 5,
                    time_spent_seconds: 0,
                    new_cards: 0,
                    learned_cards: 0,
                    locked_in_cards: 0,
                },
            )])
        };
        let totals = |view: ReviewTotalView| {
            view.points
                .iter()
                .map(|point| (point.day, point.total))
                .collect::<Vec<_>>()
        };
        let first_day = review_total_view(&history(today), today).unwrap();
        assert_eq!(totals(first_day), [(0, 0), (1, 5)]);
        let tomorrow = review_total_view(&history(today + Duration::days(1)), today).unwrap();
        assert_eq!(totals(tomorrow), [(0, 0), (1, 5)]);
        assert!(review_total_view(&BTreeMap::new(), today).is_none());
    }
}
