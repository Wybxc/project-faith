module.exports = {
  apps: [
    {
      script: 'bun dev',
      autorestart: false,
      shutdown_with_message: true,
    },
    {
      script: './scripts/protogen.sh',
      watch: '../proto',
    },
  ],
};
