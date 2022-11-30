async function generateAwsEc2JSON(input) {
    // AWS CLI encodes to base64, so to avoid double encoding the butane UserData we decode it here
    // But the user could provide something manually that is not in base64, so we have a hard time handling it here
    // We could have just forced butane to output UserData without using base64,
    // but that doesn't work well with string props, so we hacked this around and will need to revisit
    if (input.domain.UserData) {
        const base64 = await siExec.waitUntilEnd("base64", ["-d"], {input: input.domain.UserData});
        if (base64.exitCode === 0) {
            input.domain.UserData = base64.stdout;
        }
    }

    // Initialize the input JSON.
    const object = {
        "ImageId": input.domain.ImageId,
        "InstanceType": input.domain.InstanceType,
        "KeyName": input.domain.KeyName,
        "SecurityGroupIds": input.domain.SecurityGroupIds,
        "UserData": input.domain.UserData,
    };

    // Normalize tags to be in the weird Map-like structure AWS uses (array of { Key: string, Value: string } where Key is unique
    const tags = [];
    if (input.domain.tags) {
        for (const [key, value] of Object.entries(input.domain.tags)) {
            tags.push({
                "Key": key,
                "Value": value,
            });
        }
        if (tags.length > 0) {
            object["TagSpecifications"] = [{
                "ResourceType": input.domain.awsResourceType,
                "Tags": tags
            }];
        }
    }

    // FIXME(nick): once the bug related to child fields for complex objects is fixed, return the format too.
    // return {
    //     format: "json",
    //     code: JSON.stringify(object, null, '\t'),
    // };
    return JSON.stringify(object, null, '\t');
}