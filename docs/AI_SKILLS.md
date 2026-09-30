# NekoDroid AI Skill Format

AI skills are local JSON manifests stored under the NekoDroid data directory in `skills/`.

A skill uses schema version 1 and contains:

- `id`, `name`, and `version`;
- Android package names used to match a game/app;
- supported orientations;
- named virtual-control definitions;
- named UI regions;
- a per-game system prompt;
- an optional repository URL for discovery/distribution metadata.

Example repository index:

```json
{
  "schemaVersion": 1,
  "name": "Example NekoDroid Skill Repository",
  "skills": [
    {
      "id": "example-game",
      "version": "1.0.0",
      "manifestUrl": "https://example.invalid/skills/example-game.json"
    }
  ]
}
```

Repository indexes are metadata only. NekoDroid should validate imported manifests locally and must not execute arbitrary code from a skill. Controls are declarative and are mapped onto NekoDroid's existing virtual Android input actions.

The built-in `generic-android` skill provides basic tap/back/home controls. Custom skills can be created in the UI or imported/exported as JSON. Runtime game-state extraction reports the foreground package/activity, current orientation, and display size for skill selection and prompts.
