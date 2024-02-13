# upload
A simple webserver to upload and download files.

## Running
Run upload with docker:
```
docker run --name upload -v ./upload:/upload -v ./config:/config -p 80:80 ghcr.io/adridevelopsthings/upload:main
```
(put the `auth.toml` file in the `./config` directory)

or

build it yourself:

```
cargo build --release && ./target/release/upload
```

### Environment variables
- `UPLOAD_DIRECTORY`: The directory where the uploaded files should be put in (default `upload`, docker default `/upload`)
- `AUTH_CONFIG_PATH`: The path to your `auth.toml` file (default `auth.toml`, docker default `/config/auth.toml`)
- `LISTEN_ADDRESS`: The address where the webserver should listen (default `127.0.0.1:3000`, docker default `0.0.0.0:80`)

# auth.toml
```toml
default_auth_scheme = "Basic" # not required, this is the default value
default_max_filesize = 10737418240 # 10 GB, not required, this is the default value
allow_downloading_for_everyone = false # not required, this is the default value
allow_uploading_for_everyone = false # not required, this is the default value

[[basic]] # a basic authorization method
username = "username" # required
password = "bcrypt hashed password" # required
max_filesize = 1024 # not required, uses `default_max_filesize` as default
allow_download = true # not required, this is the default value
allow_upload = true # not required, this is the default value

[[bearer]]
secret = "PLEASE USE A SAFE ONE" # required, jsonwebtoken HS256 secret, create with `openssl rand -hex 64`
default_max_filesize = 1024 # not required, uses `default_max_filesize` as defualt
default_permissions = ["download", "upload"] # not required, this is the default value
max_filesize_field_name = "max_filesize" # the name of the field inside the json containing the max_filesize, not required, this is the default value
permissions_field_name = "permissions" # the name of the field inside the json containing the permissions, not required, this is the default value
```

# HTTP
## Uploading
Request:
```
POST /upload/filename.txt HTTP/1.1
Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
Content-Length: 81

This is the body of the http message and contains the content you want to upload.
```
Response:
```
HTTP/1.1 201 Created
Location: /d/md5hash_filename.txt
Content-Length: 23

/d/md5hash_filename.txt
```

## Downloading
Request:
```
GET /d/md5hash_filename.txt HTTP/1.1
Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
```
Response:
```
HTTP/1.1 200 OK
Content-Length: 74
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="md5hash_filename.txt"

This is the body of the http message and contains the content of the file.
```