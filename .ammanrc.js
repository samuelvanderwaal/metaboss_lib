module.exports = {
  validator: {
    killRunningValidators: true,
    programs: [
      {
        label: "Token Metadata Program",
        programId: "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
        deployPath: "tests/fixtures/mpl_token_metadata.so"
      }
    ]
  }
};
