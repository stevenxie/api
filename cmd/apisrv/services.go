package main

import (
	"os"

	"github.com/dmksnnk/sentryhook"
	raven "github.com/getsentry/raven-go"

	"github.com/sirupsen/logrus"
	"github.com/stevenxie/api/internal/info"
	ess "github.com/unixpickle/essentials"
)

func buildRaven() *raven.Client {
	rc, err := raven.New(os.Getenv("SENTRY_DSN"))
	if err != nil {
		ess.Die("Failed to build Raven client:", err)
	}

	// Configure client.
	if env := os.Getenv("GOENV"); env != "" {
		rc.SetEnvironment(env)
	}
	rc.SetRelease(info.Version)

	return rc
}

// buildLogger builds an application-level logger, which also captures errors
// using Sentry.
func buildLogger(rc *raven.Client) *logrus.Logger {
	log := logrus.New()
	log.SetOutput(os.Stdout)

	// Set logger level.
	if os.Getenv("GOENV") == "development" {
		log.SetLevel(logrus.DebugLevel)
	}

	// Integrate error reporting with Sentry.
	hook := sentryhook.New(rc)
	hook.SetAsync(logrus.ErrorLevel)
	hook.SetSync(logrus.PanicLevel, logrus.FatalLevel)
	log.AddHook(hook)

	return log
}
