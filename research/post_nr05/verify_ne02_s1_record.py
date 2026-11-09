"""Import-compatible wrapper for the executable recorded-result verifier."""
import importlib.util
from pathlib import Path
PATH=Path(__file__).resolve().parents[1]/'post-nr05'/'verify_ne02_s1_record.py'
SPEC=importlib.util.spec_from_file_location('ne02_s1_recorded',PATH)
checker=importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)
verify=checker.verify
