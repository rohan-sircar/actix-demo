// pub async fn validator(
//     req: ServiceRequest,
//     credentials: BasicAuth,
// ) -> Result<ServiceRequest, Error> {
//     println!("{}", credentials.user_id());
//     println!("{:?}", credentials.password());
//     // verify credentials from db
//     let config = req.app_data::<AppConfig>().expect("Error getting config");

//     let valid =
//         web::block(move || validate_basic_auth(credentials, config)).await?;
//     if valid {
//         debug!("Success");
//         Ok(req)
//     } else {
//         println!("blah");
//         let err: Error = crate::errors::DomainError::new_password_error(
//             "Wrong password or account does not exist".to_string(),
//         )
//         .into();

//         Err(err)
//     }
// }
