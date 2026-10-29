from app.clients import ld_client


def is_on(name):
    """The team's wrapper over LaunchDarkly."""
    return ld_client.variation(name, {"key": "user"}, False)
