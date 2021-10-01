use chrono::{DateTime, NaiveDate, Utc};
use kingtime::daily_workings::timerecord::{Code, Request, TimeRecord};

fn today() -> NaiveDate {
    Utc::today().naive_local()
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

async fn get_my_timerecords(date: NaiveDate) -> Vec<TimeRecord> {
    let token = std::env::var("KINGTIME_ACCESS_TOKEN").unwrap();
    let key = std::env::var("KINGTIME_KEY").unwrap();

    let resp = kingtime::daily_workings::timerecord::get(&token, &[&key], date, date)
        .await
        .unwrap();

    let mut dws = resp.0;
    assert_eq!(dws.len(), 1);
    let mut dw = dws.remove(0);
    assert_eq!(dw.daily_workings.len(), 1);
    let dw = dw.daily_workings.remove(0);
    assert_eq!(dw.date, date);
    assert_eq!(dw.employee_key, key);
    let mut trs = dw.time_record;
    trs.sort_by_key(|record| record.time);
    trs
}

async fn timecard(code: Code) {
    let token = std::env::var("KINGTIME_ACCESS_TOKEN").unwrap();
    let key = std::env::var("KINGTIME_KEY").unwrap();

    let req = Request {
        date: today(),
        time: now(),
        code,
    };
    println!("{}", serde_json::to_string_pretty(&req).unwrap());
    let _resp = kingtime::daily_workings::timerecord::post(&token, &key, &req)
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    match &*args[1] {
        "status" => {
            let trs = get_my_timerecords(today()).await;
            if trs.is_empty() {
                println!("not at work (yet)");
                return;
            }
            match trs[trs.len() - 1].code {
                kingtime::daily_workings::timerecord::Code::In
                | kingtime::daily_workings::timerecord::Code::BreakEnd => {
                    println!("🕴 at work");
                }
                kingtime::daily_workings::timerecord::Code::Out
                | kingtime::daily_workings::timerecord::Code::BreakStart => {
                    println!("finished the work (or have a break)");
                }
            }
        }
        "in" => timecard(Code::In).await,
        "out" => timecard(Code::Out).await,
        "ls" => {
            unimplemented!();
        }
        cmd => {
            panic!("unknown command: {}", cmd);
        }
    }
}
