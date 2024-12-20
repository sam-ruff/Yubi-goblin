const branches = [
    'main',
    {
        name: 'alpha',
        prerelease: true
    }
];
const git = '@semantic-release/git';

const exec = [
    "@semantic-release/exec",
    {
        "prepareCmd": "echo \"RELEASE_VERSION=${nextRelease.version}\" >> $GITHUB_ENV"
    }
];

module.exports = {
    branches,
    plugins: [
        [
            "@semantic-release/commit-analyzer",
            {
                preset: "angular",
                releaseRules: [
                    { type: "breaking", release: "major" },
                ],
                parserOpts: {
                    noteKeywords: ["BREAKING CHANGE", "BREAKING CHANGES", "breaking:"]
                }
            }
        ],
        git,
        exec
    ]
};