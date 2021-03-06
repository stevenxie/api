use super::common::*;
use crate::models::Contact as ContactModel;

use chrono_humanize::{
    Accuracy as HumanAccuracy, HumanTime, Tense as HumanTense,
};
use chrono_tz::Tz;

use graphql::Subscription;
use std::time::Duration;
use tokio::time::interval;

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn my_age(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The time zone of the viewer.", default = "UTC")]
        time_zone: String,
    ) -> FieldResult<impl Stream<Item = FieldResult<String>>> {
        let me = ctx.data::<ContactModel>()?;
        let time_zone = Tz::from_str(&time_zone)
            .map_err(|message| format_err!(message))
            .context("failed to parse time zone")
            .into_field_result()?;
        let birthday = me
            .birthday_in_time_zone(time_zone)
            .context("failed to represent birthday in the given timezone")
            .into_field_result()?
            .and_hms(0, 0, 0);
        let stream = interval(Duration::from_secs(1)).map(move |_| {
            let now: DateTime<Tz> = Utc::now().with_timezone(&time_zone);
            let age = now - birthday;
            let age = HumanTime::from(age)
                .to_text_en(HumanAccuracy::Rough, HumanTense::Present);
            Ok(age)
        });
        Ok(stream)
    }
}
