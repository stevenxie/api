package server

import (
	"context"
	"io/ioutil"

	errors "golang.org/x/xerrors"

	echo "github.com/labstack/echo/v4"
	"github.com/rs/zerolog"
	"github.com/stevenxie/api/pkg/about"
	"github.com/stevenxie/api/pkg/metrics"
	"github.com/stevenxie/api/pkg/music"
)

// Server serves the accounts REST API.
type Server struct {
	echo   *echo.Echo
	logger zerolog.Logger

	info             about.InfoStore
	productivity     metrics.ProductivityService
	currentlyPlaying music.CurrentlyPlayingService
}

// New creates a new Server.
func New(
	info about.InfoStore,
	productivity metrics.ProductivityService,
	currentlyPlaying music.CurrentlyPlayingService,
	l zerolog.Logger,
) *Server {
	echo := echo.New()
	echo.Logger.SetOutput(ioutil.Discard) // disable logger

	return &Server{
		echo:             echo,
		logger:           l,
		info:             info,
		productivity:     productivity,
		currentlyPlaying: currentlyPlaying,
	}
}

// ListenAndServe listens and serves on the specified address.
func (srv *Server) ListenAndServe(addr string) error {
	if addr == "" {
		return errors.New("server: addr must be non-empty")
	}

	// Register routes.
	if err := srv.registerRoutes(); err != nil {
		return errors.Errorf("server: registering routes: %w", err)
	}

	// Listen for connections.
	srv.logger.Info().Str("addr", addr).Msg("Listening for connections...")
	return srv.echo.Start(addr)
}

// Shutdown shuts down the server gracefully without interupting any active
// connections.
func (srv *Server) Shutdown(ctx context.Context) error {
	return srv.echo.Shutdown(ctx)
}
