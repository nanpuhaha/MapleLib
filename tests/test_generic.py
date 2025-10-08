import os
import pytest
from maple_rust import WzFile, WzMapleVersion

# --- Generic Tests ---

def test_wzfile_creation():
    """Tests that a WzFile object can be created without errors."""
    try:
        wz_file = WzFile("non_existent_file.wz", WzMapleVersion.GMS)
        assert wz_file is not None
        assert wz_file.path == "non_existent_file.wz"
        assert wz_file.maple_version is not None
    except Exception as e:
        pytest.fail(f"WzFile creation failed with an unexpected exception: {e}")

def test_parse_non_existent_file():
    """Tests that parsing a non-existent file raises an IOError."""
    wz_file = WzFile("non_existent_file.wz", WzMapleVersion.GMS)
    with pytest.raises(IOError):
        wz_file.parse()

# --- File-Based Test Template ---

# To use this test, replace "YOUR_WZ_FILE.wz" with the path to your .wz file.
# You may also need to adjust the WzMapleVersion if your file is not from GMS.
# This test is disabled by default to prevent errors. Remove the @pytest.mark.skip decorator to run it.

WZ_FILE_PATH = "YOUR_WZ_FILE.wz"

@pytest.mark.skipif(not os.path.exists(WZ_FILE_PATH) or WZ_FILE_PATH == "YOUR_WZ_FILE.wz", reason="WZ file not provided")
def test_parse_real_wz_file():
    """
    This is a template for testing with a real .wz file.
    1. Place your .wz file in the `maple-rust/` directory.
    2. Replace "YOUR_WZ_FILE.wz" with the actual file name.
    3. Remove the `@pytest.mark.skipif(...)` decorator above this function.
    """
    assert os.path.exists(WZ_FILE_PATH), f"File not found: {WZ_FILE_PATH}"

    wz_file = WzFile(WZ_FILE_PATH, WzMapleVersion.GMS)

    try:
        wz_file.parse()
    except Exception as e:
        pytest.fail(f"Parsing '{WZ_FILE_PATH}' failed with an unexpected exception: {e}")

    assert wz_file.wz_dir is not None, "The wz_dir should not be None after parsing."

    # You can add more specific assertions here. For example:
    # first_image = wz_file.wz_dir.images[0]
    # assert first_image.name == "expected_image_name.img"

    print(f"\nSuccessfully parsed '{WZ_FILE_PATH}'.")
    print(f"Version: {wz_file.version}")
    print(f"Directory Name: {wz_file.wz_dir.name}")
    print(f"Found {len(wz_file.wz_dir.images)} images and {len(wz_file.wz_dir.directories)} directories at the root.")