
# Run this to build a local copy of the container
# docker build -t schedmap_build_container .
# Run this to restore a saved .tar containing a build container
# docker load -i c:/path/to/schedmap_container.tar

docker run -ti -v ${pwd}:/src/ -p 8000:8000 -p 8001:8001 schedmap_build_container cargo run


