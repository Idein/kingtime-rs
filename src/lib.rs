use reqwest::header::{self, HeaderMap};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Response<R> {
    Error { errors: Vec<ErrorData> },
    Ok(R),
}

#[derive(Debug, Deserialize)]
pub struct ErrorData {
    message: String,
    code: u32,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("{0:?}")]
    Api(Vec<ErrorData>),
}

pub type Result<T> = std::result::Result<T, Error>;

async fn get<D: DeserializeOwned>(access_token: &str, api: &str) -> Result<D> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::AUTHORIZATION,
        format!("Bearer {}", access_token).parse().unwrap(),
    );

    let resp: Response<D> = reqwest::Client::new()
        .get(api)
        .headers(headers)
        .send()
        .await?
        .json()
        .await?;
    match resp {
        Response::Error { errors } => Err(Error::Api(errors)),
        Response::Ok(data) => Ok(data),
    }
}

async fn post<S: Serialize + ?Sized, D: DeserializeOwned>(
    access_token: &str,
    api: &str,
    payload: &S,
) -> Result<D> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::AUTHORIZATION,
        format!("Bearer {}", access_token).parse().unwrap(),
    );

    let resp: Response<D> = reqwest::Client::new()
        .post(api)
        .headers(headers)
        .json(payload)
        .send()
        .await?
        .json()
        .await?;
    match resp {
        Response::Error { errors } => Err(Error::Api(errors)),
        Response::Ok(data) => Ok(data),
    }
}

pub mod daily_workings {
    use super::Result;
    use serde::Deserialize;

    pub async fn get(access_token: &str) -> Result<Response> {
        super::get(access_token, "https://api.kingtime.jp/v1.0/daily-workings").await
    }

    #[derive(Debug, Deserialize)]
    pub struct Response(Vec<DailyWorkings>);

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DailyWorkings {
        date: String,
        daily_workings: Vec<DailyWorking>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DailyWorking {
        date: String,
        employee_key: String,
        // ...
    }

    #[test]
    fn deserialize_response() {
        let ex = r##"
[
  {
    "date": "2016-05-01",
    "dailyWorkings": [
      {
        "date": "2016-05-01",
        "employeeKey": "8b6ee646a9620b286499c3df6918c4888a97dd7bbc6a26a18743f4697a1de4b3",
        "currentDateEmployee": {
          "divisionCode": "1000",
          "divisionName": "本社",
          "gender": "male",
          "typeCode": "1",
          "typeName": "正社員",
          "code": "1000",
          "lastName": "勤怠",
          "firstName": "太郎",
          "lastNamePhonetics": "キンタイ",
          "firstNamePhonetics": "タロウ",
          "employeeGroups": [
            {
              "code": "0001",
              "name": "人事部"
            },
            {
              "code": "0002",
              "name": "総務部"
            }
          ]
        },
        "workPlaceDivisionCode": "1000",
        "workPlaceDivisionName": "本社",
        "isClosing": true,
        "isHelp": false,
        "isError": false,
        "workdayTypeName": "平日",
        "assigned": 480,
        "unassigned": 135,
        "overtime": 135,
        "lateNight": 0,
        "lateNightUnassigned": 0,
        "lateNightOvertime": 0,
        "breakTime": 60,
        "late": 0,
        "earlyLeave": 0,
        "totalWork": 615,
        "holidaysObtained": {
          "fulltimeHoliday": {
            "code": 1,
            "name": "有休"
          },
          "halfdayHolidays": [
            {
              "typeName": "PM休",
              "code": 1,
              "name": "有休"
            }
          ],
          "hourHolidays": [
            {
              "start": "2016-05-01T10:00:00+09:00",
              "end": "2016-05-01T11:00:00+09:00",
              "minutes": 60,
              "code": 1,
              "name": "有休"
            }
          ]
        },
        "autoBreakOff": 1,
        "discretionaryVacation": 0,
        "customDailyWorkings": [
          {
            "code": "dCus1",
            "name": "日別カスタム1",
            "calculationUnitCode": 1,
            "calculationResult": 1
          },
          {
            "code": "dCus2",
            "name": "日別カスタム2",
            "calculationUnitCode": 2,
            "calculationResult": 10
          },
          {
            "code": "dCus3",
            "name": "日別カスタム3",
            "calculationUnitCode": 4,
            "calculationResult": 100
          }
        ]
      }
    ]
  }
]
        "##;

        let _: Response = serde_json::from_str(ex).unwrap();
    }

    pub mod timerecord {
        use crate::Result;
        use serde::{Deserialize, Serialize};

        pub async fn post(access_token: &str, key: &str, req: &Request) -> Result<()> {
            let Response {} = crate::post(
                access_token,
                &format!(
                    "https://api.kingtime.jp/v1.0/daily-workings/timerecord/{}",
                    key
                ),
                req,
            )
            .await?;
            Ok(())
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Request {
            date: String,
            time: String,
        }

        #[test]
        fn serialize_request() {
            let req = Request {
                date: "2016-05-01".to_owned(),
                time: "2016-05-01T09:00:00+09:00".to_owned(),
            };

            let json = r##"
            {
                "date": "2016-05-01",
                "time": "2016-05-01T09:00:00+09:00",
            }
            "##;

            let v1 = serde_json::from_str::<serde_json::Value>(json).unwrap();
            let v2 =
                serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&req).unwrap())
                    .unwrap();

            assert_eq!(v1, v2);
        }

        #[derive(Deserialize)]
        pub struct Response {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let token = std::env::var("KINGTIME_ACCESS_TOKEN").unwrap();
        println!("{:?}", daily_workings::get(&token).await.unwrap());
    }
}
