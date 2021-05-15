Testing out the Rust framework Actix-Web to create a JSON API CRUD Web App.

## API Demo

### Get Users

```
curl -X GET http://localhost:7800/api/users
```

```
{
  "name": "user1",
  "registration_date": "2020-05-09T06:17:26"
}
```

```
curl -X GET http://localhost:7800/api/users
```

```
[
  {
    "name": "user1",
    "registration_date": "2020-05-09T06:17:26"
  },
  {
    "name": "user2",
    "registration_date": "2020-05-12T12:43:13"
  },
  {
    "name": "user3",
    "registration_date": "2020-05-15T07:47:50"
  }
]
```

### Create User

```
curl -H "content-type: application/json" \
-X POST \
-i http://localhost:7800/api/users \
--data '{"name":"user4","password":"test"}'
```

```
[
  {
    "name": "user1",
    "registration_date": "2020-05-09T06:17:26"
  },
  {
    "name": "user2",
    "registration_date": "2020-05-12T12:43:13"
  },
  {
    "name": "user3",
    "registration_date": "2020-05-15T07:47:50"
  },
  {
    "name": "user4",
    "registration_date": "2020-08-01T05:04:05"
  }
]
```

### DTO Validation

```
curl -H "content-type: application/json" \
-X POST \
-i http://localhost:7800/api/users \
--data '{"name":"abc","password":"test"}' # min length for name is 4
```

```
ValidationErrors({"name": Field([ValidationError { code: "length", message: None, params: {"value": String("abc"), "min": Number(4), "max": Number(10)} }])})
```

## Memory Usage

Memory usage as compared to interpreted languages was my primary motivation for looking into rust as a backend language. As of writing, the demo app uses less than 50MB of memory.

## License

AGPLv3
