/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU Affero General Public License as
* published by the Free Software Foundation, either version 3 of the
* License, or (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU Affero General Public License for more details.
*
* You should have received a copy of the GNU Affero General Public License
* along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use actix_web::http::{header, StatusCode};
use actix_web::test;

use crate::api::v1::auth::*;
use crate::data::Data;
use crate::errors::*;
use crate::*;

use crate::tests::*;

#[actix_rt::test]
async fn auth_works() {
    let data = Data::new().await;
    const NAME: &str = "testuser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testuser1@a.com";
    const SIGNIN: &str = "/api/v1/signin";
    const SIGNUP: &str = "/api/v1/signup";
    const GET_SECRET: &str = "/api/v1/account/secret/";

    let mut app = get_app!(data).await;

    delete_user(NAME, &data).await;

    // 1. Register with email == None
    let msg = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        email: None,
    };
    let resp = test::call_service(&mut app, post_request!(&msg, SIGNUP).to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    // delete user
    delete_user(NAME, &data).await;

    // 1. Register and signin
    let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);

    // chech if get user secret works
    let resp = test::call_service(
        &mut app,
        test::TestRequest::get()
            .cookie(cookies.clone())
            .uri(GET_SECRET)
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // check if update user secret works
    let resp = test::call_service(
        &mut app,
        test::TestRequest::post()
            .cookie(cookies.clone())
            .uri(GET_SECRET)
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // 2. check if duplicate username is allowed
    let msg = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        email: Some(EMAIL.into()),
    };
    bad_post_req_test(
        NAME,
        PASSWORD,
        SIGNUP,
        &msg,
        ServiceError::UsernameTaken,
        StatusCode::BAD_REQUEST,
    )
    .await;

    // 3. sigining in with non-existent user
    let mut login = Login {
        username: "nonexistantuser".into(),
        password: msg.password.clone(),
    };
    bad_post_req_test(
        NAME,
        PASSWORD,
        SIGNIN,
        &login,
        ServiceError::UsernameNotFound,
        StatusCode::NOT_FOUND,
    )
    .await;

    // 4. trying to signin with wrong password
    login.username = NAME.into();
    login.password = NAME.into();

    bad_post_req_test(
        NAME,
        PASSWORD,
        SIGNIN,
        &login,
        ServiceError::WrongPassword,
        StatusCode::UNAUTHORIZED,
    )
    .await;

    // 5. signout
    let signout_resp = test::call_service(
        &mut app,
        test::TestRequest::get()
            .uri("/logout")
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(signout_resp.status(), StatusCode::OK);
    let headers = signout_resp.headers();
    assert_eq!(headers.get(header::LOCATION).unwrap(), "/login");
}

#[actix_rt::test]
async fn email_udpate_and_del_userworks() {
    const NAME: &str = "testuser2";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "testuser1@a.com2";
    const DEL_URL: &str = "/api/v1/account/delete";
    const EMAIL_UPDATE: &str = "/api/v1/account/email/";

    {
        let data = Data::new().await;
        delete_user(NAME, &data).await;
    }

    let (data, creds, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let mut app = get_app!(data).await;

    let email_payload = Email {
        email: EMAIL.into(),
    };
    let email_update_resp = test::call_service(
        &mut app,
        post_request!(&email_payload, EMAIL_UPDATE)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;

    assert_eq!(email_update_resp.status(), StatusCode::OK);

    let payload = Password {
        password: creds.password,
    };

    let delete_user_resp = test::call_service(
        &mut app,
        post_request!(&payload, DEL_URL)
            .cookie(cookies)
            .to_request(),
    )
    .await;

    assert_eq!(delete_user_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn uname_email_exists_works() {
    const NAME: &str = "testuserexists";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "testuserexists@a.com2";
    const UNAME_CHECK: &str = "/api/v1/account/username/exists";
    const EMAIL_CHECK: &str = "/api/v1/account/email/exists";

    {
        let data = Data::new().await;
        delete_user(NAME, &data).await;
    }

    let (data, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let mut app = get_app!(data).await;

    let mut payload = AccountCheckPayload { val: NAME.into() };

    let user_exists_resp = test::call_service(
        &mut app,
        post_request!(&payload, UNAME_CHECK)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(user_exists_resp.status(), StatusCode::OK);
    let mut resp: AccountCheckResp = test::read_body_json(user_exists_resp).await;
    assert!(resp.exists);

    payload.val = PASSWORD.into();

    let user_doesnt_exist = test::call_service(
        &mut app,
        post_request!(&payload, UNAME_CHECK)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(user_doesnt_exist.status(), StatusCode::OK);
    resp = test::read_body_json(user_doesnt_exist).await;
    assert!(!resp.exists);

    let email_doesnt_exist = test::call_service(
        &mut app,
        post_request!(&payload, EMAIL_CHECK)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(email_doesnt_exist.status(), StatusCode::OK);
    resp = test::read_body_json(email_doesnt_exist).await;
    assert!(!resp.exists);

    payload.val = EMAIL.into();

    let email_exist = test::call_service(
        &mut app,
        post_request!(&payload, EMAIL_CHECK)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(email_exist.status(), StatusCode::OK);
    resp = test::read_body_json(email_exist).await;
    assert!(resp.exists);
}
