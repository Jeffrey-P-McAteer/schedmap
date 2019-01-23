
docker build -t schedmap_build_container .

docker run -ti -v ${pwd}:/src/ -p 8000:8000 schedmap_build_container cargo run


