async function qualification(component) {
  const code = component.data.properties.code["si:generateAwsEipJSON"]?.code;
  if (!code) {
    return {
      qualified: false,
      message: "Component doesn't have JSON representation",
    };
  }

  if (!component.data.properties.domain.region) {
    return {
      qualified: false,
      message: "Component doesn't have a region set",
    };
  }

  // Now, dry-run creation of the elastic ip
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "allocate-address",
    "--region",
    component.data.properties.domain.region,
    "--domain",
    "vpc",
    "--dry-run",
    "--cli-input-json",
    code,
  ]);

  // We have to use `includes` instead of `startsWith` because the line can start with a line feed char
  const success = child.stderr.includes("An error occurred (DryRunOperation)");

  return {
    qualified: success,
    message: success ? "Component qualified" : child.stderr,
  };
}