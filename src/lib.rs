use reqwest::header::{self, HeaderMap};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

// KoT API only correctly recognizes iso8061 strings with +09:00
mod ts_seconds_jst {
    use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
    use serde::ser::Serializer;
    use serde::Serialize;

    pub fn serialize<S>(value: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str = value.to_rfc3339_opts(SecondsFormat::Secs, false);
        let value: DateTime<Utc> = str.parse().unwrap();
        let datetime = value + FixedOffset::east(9 * 3600);
        datetime.to_rfc3339().serialize(serializer)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Response<R> {
    Error { errors: Vec<ErrorData> },
    Ok(R),
}

#[derive(Debug, Deserialize)]
pub struct ErrorData {
    pub message: String,
    pub code: u32,
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

async fn get_with_query<D: DeserializeOwned>(
    access_token: &str,
    api: &str,
    query: &impl Serialize,
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
        .get(api)
        .headers(headers)
        .query(query)
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

pub mod employees {
    use super::Result;
    use serde::Deserialize;

    pub async fn get(access_token: &str, code: &str) -> Result<Response> {
        crate::get(
            access_token,
            &format!("https://api.kingtime.jp/v1.0/employees/{}", code),
        )
        .await
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub last_name: String,
        pub first_name: String,
        pub key: String,
    }
}

pub mod daily_workings {
    use super::Result;
    use chrono::NaiveDate;
    use serde::Deserialize;

    pub async fn get(access_token: &str) -> Result<Response> {
        super::get(access_token, "https://api.kingtime.jp/v1.0/daily-workings").await
    }

    #[derive(Debug, Deserialize)]
    pub struct Response(pub Vec<DailyWorkings>);

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DailyWorkings {
        pub date: NaiveDate,
        pub daily_workings: Vec<DailyWorking>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DailyWorking {
        pub date: NaiveDate,
        pub employee_key: String,
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
        use chrono::{DateTime, NaiveDate, Utc};
        use serde::{de::Visitor, Deserialize, Serialize};

        pub async fn post(access_token: &str, key: &str, req: &Request) -> Result<()> {
            let PostResponse {} = crate::post(
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
            pub date: NaiveDate,
            #[serde(with = "crate::ts_seconds_jst")]
            pub time: DateTime<Utc>,
            pub code: Code,
        }

        #[test]
        fn serialize_request() {
            let req = Request {
                date: "2016-05-01".parse().unwrap(),
                time: "2016-05-01T09:00:00+09:00".parse().unwrap(),
                code: Code::BreakEnd,
            };

            let json = r##"
            {
                "date": "2016-05-01",
                "time": "2016-05-01T00:00:00Z",
                "code": "4"
            }
            "##;

            let v1 = serde_json::from_str::<serde_json::Value>(json).unwrap();
            let v2 =
                serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&req).unwrap())
                    .unwrap();

            assert_eq!(v1, v2);
        }

        #[derive(Deserialize)]
        struct PostResponse {}

        pub async fn get(
            access_token: &str,
            keys: &[&str],
            start: NaiveDate,
            end: NaiveDate,
        ) -> Result<Response> {
            crate::get_with_query(
                access_token,
                "https://api.kingtime.jp/v1.0/daily-workings/timerecord",
                &[
                    ("employeeKeys", &*keys.join(",")),
                    ("start", &start.to_string()),
                    ("end", &end.to_string()),
                ],
            )
            .await
        }

        #[derive(Debug, Deserialize)]
        pub struct Response(pub Vec<DailyWorkings>);

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct DailyWorkings {
            pub date: NaiveDate,
            pub daily_workings: Vec<DailyWorking>,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct DailyWorking {
            pub date: NaiveDate,
            pub employee_key: String,
            pub time_record: Vec<TimeRecord>,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct TimeRecord {
            pub time: DateTime<Utc>,
            pub code: Code,
        }

        #[derive(Debug, Clone, Copy)]
        pub enum Code {
            In,
            Out,
            BreakStart,
            BreakEnd,
        }

        struct CodeVisitor;

        impl<'de> Visitor<'de> for CodeVisitor {
            type Value = Code;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("code must be an str")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let c = match v {
                    "1" => Code::In,
                    "2" => Code::Out,
                    "3" => Code::BreakStart,
                    "4" => Code::BreakEnd,
                    _ => return Err(E::custom(format!("unknown code: {}", v))),
                };
                Ok(c)
            }
        }

        impl<'de> Deserialize<'de> for Code {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(CodeVisitor)
            }
        }

        impl Serialize for Code {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    Code::In => serializer.serialize_str("1"),
                    Code::Out => serializer.serialize_str("2"),
                    Code::BreakStart => serializer.serialize_str("3"),
                    Code::BreakEnd => serializer.serialize_str("4"),
                }
            }
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
                      "timeRecord": [
                        {
                          "time": "2016-05-01T09:00:00+09:00",
                          "code": "1",
                          "name": "出勤",
                          "divisionCode": "1000",
                          "divisionName": "本社",
                          "latitude": 35.6672237,
                          "longitude": 139.7422207
                        },
                        {
                          "time": "2015-05-01T18:00:00+09:00",
                          "code": "2",
                          "name": "退勤",
                          "divisionCode": "1000",
                          "divisionName": "本社",
                          "credentialCode": 300,
                          "credentialName": "KOTSL",
                          "latitude": 35.6672237,
                          "longitude": 139.7422207
                        },
                        {
                          "time": "2016-05-01T10:00:00+09:00",
                          "code": "3",
                          "name": "休憩開始",
                          "divisionCode": "1000",
                          "divisionName": "本社"
                        },
                        {
                          "time": "2016-05-01T11:00:00+09:00",
                          "code": "4",
                          "name": "休憩終了",
                          "divisionCode": "1000",
                          "divisionName": "本社"
                        }
                      ]
                    }
                  ]
                }
              ]
            "##;

            let _: Response = serde_json::from_str(ex).unwrap();
        }
    }
}
