#!/usr/bin/env bats

@test "Empty settings should fail" {
	run kwctl run  --request-path test_data/pvc-fast-storage-class-request.json --settings-json '{}'  annotated-policy.wasm

	  # this prints the output when one the checks below fails
	  echo "output = ${output}"

	[ "$status" -ne 0 ]
	[ $(expr "$output" : '.*"valid":false.*') -ne 0 ]
	[ $(expr "$output" : '.*"message":"deniedStorageClasses cannot be empty".*') -ne 0 ]
}

@test "Empty denied storage classes list should fail" {
	run kwctl run  --request-path test_data/pvc-fast-storage-class-request.json --settings-json '{"deniedStorageClasses":[]}'  annotated-policy.wasm

	  # this prints the output when one the checks below fails
	  echo "output = ${output}"

	[ "$status" -ne 0 ]
	[ $(expr "$output" : '.*"valid":false.*') -ne 0 ]
	[ $(expr "$output" : '.*"message":"deniedStorageClasses cannot be empty".*') -ne 0 ]
}

@test "Fallback cannot be in the denied storage classes list" {
	run kwctl run  --request-path test_data/pvc-fast-storage-class-request.json --settings-json '{"deniedStorageClasses":["fast"], fallbackStorageClass: "fast"}'  annotated-policy.wasm

	  # this prints the output when one the checks below fails
	  echo "output = ${output}"

	[ "$status" -ne 0 ]
	[ $(expr "$output" : '.*"valid":false.*') -ne 0 ]
	[ $(expr "$output" : '.*"message":"fallbackStorageClass cannot be in deniedStorageClasses".*') -ne 0 ]
}

@test "Accept PVC not using denied storage classes names" {
	run kwctl run  --request-path test_data/pvc-fast-storage-class-request.json --settings-json '{"deniedStorageClasses": ["slow"]}'  annotated-policy.wasm

	  # this prints the output when one the checks below fails
	  echo "output = ${output}"

	[ "$status" -eq 0 ]
	[ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
}

@test "Reject PVC using denied storage classes names" {
	run kwctl run  --request-path test_data/pvc-fast-storage-class-request.json --settings-json '{"deniedStorageClasses": ["fast"]}'  annotated-policy.wasm

  	# this prints the output when one the checks below fails
  	echo "output = ${output}"

	[ $(expr "$output" : '.*"allowed":false.*') -ne 0 ]
	[ $(expr "$output" : '.*"message":"storage class \\"fast\\" is not allowed".*') -ne 0 ]
}

@test "Mutate PVC using denied storage classes names when fallback is defined" {
	run kwctl run  --request-path test_data/pvc-fast-storage-class-request.json --settings-json '{"deniedStorageClasses": ["fast"], "fallbackStorageClass": "fallback"}'  annotated-policy.wasm

	  # this prints the output when one the checks below fails
	  echo "output = ${output}"

	[ "$status" -eq 0 ]
	[ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
	[ $(expr "$output" : '.*"patchType":"JSONPatch".*') -ne 0 ]
}

