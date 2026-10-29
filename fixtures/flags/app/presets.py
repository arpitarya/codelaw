from app.clients import flags, growthbook, ld_client, openfeature, split_client, unleash

LD = ld_client.variation("ld-py", {"key": "user"}, False)
UN = unleash.is_enabled("un-py")
SPLIT = split_client.get_treatment("user", "split-py")
OF = openfeature.get_boolean_value("of-py", False)
GB = growthbook.is_on("gb-py")
FS = flags.is_feature_enabled("fs-py")
