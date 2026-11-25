import io
import tarfile
import tempfile
import unittest

from android.extract_ld_linux import extract_member


class TestExtractLdLinux(unittest.TestCase):
    def test_extract_member_reads_tar_xz(self):
        with tempfile.NamedTemporaryFile(suffix=".tar.xz") as tmp:
            with tarfile.open(tmp.name, "w:xz") as tf:
                payload = b"dummy-so-bytes"
                info = tarfile.TarInfo("lib/ld-linux.so")
                info.size = len(payload)
                tf.addfile(info, io.BytesIO(payload))

            self.assertEqual(extract_member(tmp.name, "lib/ld-linux.so"), payload)

    def test_extract_member_missing_raises(self):
        with tempfile.NamedTemporaryFile(suffix=".tar.xz") as tmp:
            with tarfile.open(tmp.name, "w:xz"):
                pass

            with self.assertRaises((KeyError, RuntimeError)):
                extract_member(tmp.name, "missing.so")


if __name__ == "__main__":
    unittest.main()
