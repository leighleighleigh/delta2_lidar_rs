from setuptools import setup, find_namespace_packages
from setuptools_rust import Binding, RustExtension
from pathlib import Path
import toml

def get_version() -> str:
    # this makes the python version match the cargo version :)
    version = {}
    # the Cargo.toml and setup.py file are in the same directory,
    # so we can use the __file__ variable to get the path to the Cargo.toml file.
    cargo_toml_path = Path(__file__).parent / "Cargo.toml"
    # Open the toml file and retrieve the [package].version value.
    with open(cargo_toml_path, "r") as f:
        cargo_toml = toml.load(f)
        version = cargo_toml["package"]["version"]

    return version

setup(
    name="delta2-lidar",
    version=get_version(),
    packages=["delta2_lidar"],
    zip_safe=False,
    rust_extensions=[
        RustExtension(
            "delta2_lidar.delta2_lidar_py",
            path="Cargo.toml",
            binding=Binding.PyO3,
            py_limited_api='auto'
        )
    ],
    include_package_data=True,
)
