.PHONY: verify check-receipt

verify:
	bash scripts/verify.sh

check-receipt:
	python3 scripts/check_scope.py
	python3 scripts/validate_formal_receipt.py formal-verification-receipt.json --base .
