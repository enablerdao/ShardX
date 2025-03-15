from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="shardx-sdk",
    version="0.1.0",
    author="ShardX Team",
    author_email="info@shardx.io",
    description="Python SDK for ShardX blockchain platform",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/enablerdao/ShardX",
    packages=find_packages(),
    classifiers=[
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.7",
    install_requires=[
        "requests>=2.25.0",
        "pycryptodome>=3.10.0",
        "base58>=2.1.0",
        "typing-extensions>=4.0.0",
    ],
)