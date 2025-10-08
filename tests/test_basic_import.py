import maple_rust

def test_import_wzfile():
    from maple_rust import WzFile
    assert WzFile is not None

def test_import_wzmapleversion():
    from maple_rust import WzMapleVersion
    assert WzMapleVersion is not None
    assert WzMapleVersion.GMS is not None