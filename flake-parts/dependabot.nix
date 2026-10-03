let
  configPath = ".github/dependabot.yml";
  automergeWorkflowPath = ".github/workflows/dependabot-automerge.yml";
in
{
  perSystem =
    { pkgs, ... }:
    {
      files.file = {
        ${automergeWorkflowPath}.source = pkgs.writers.writeJSON "dependabot-automerge.yml" {
          jobs.dependabot = {
            "if" =
              "github.event.pull_request.user.login == 'dependabot[bot]' && github.repository == 'molybdenumsoftware/statix'";
            runs-on = "ubuntu-latest";

            steps = [
              {
                "with".github-token = "\${{ secrets.GITHUB_TOKEN }}";
                id = "metadata";
                name = "Dependabot metadata";
                uses = "dependabot/fetch-metadata@main";
              }
              {

                env = {

                  GH_TOKEN = "\${{secrets.GITHUB_TOKEN}}";
                  PR_URL = "\${{github.event.pull_request.html_url}}";
                };

                name = "Enable auto-merge for Dependabot PRs";
                run = ''gh pr merge --auto --merge "$PR_URL"'';
              }
            ];
          };

          name = "Dependabot auto-merge";

          on = "pull_request";

          permissions = {
            contents = "write";
            pull-requests = "write";
          };
        };

        ${configPath}.source = pkgs.writers.writeJSON "dependabot.yml" {
          updates = [
            {
              commit-message = {
                include = "scope";
                prefix = "chore";
              };

              directory = "/";
              package-ecosystem = "cargo";
              schedule.interval = "daily";
            }
            {
              commit-message.prefix = "chore";
              directory = "/";
              package-ecosystem = "nix";
              schedule.interval = "daily";
            }
          ];

          version = 2;
        };
      };

      treefmt.settings.global.excludes = [
        automergeWorkflowPath
        configPath
      ];
    };
}
