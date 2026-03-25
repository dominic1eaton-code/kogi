// api/server/server.go — HTTP server and router setup
package server

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/kogi/api/handlers"
	"github.com/kogi/api/middleware"
)

// Config holds server configuration.
type Config struct {
	Port         string
	ReadTimeout  time.Duration
	WriteTimeout time.Duration
	IdleTimeout  time.Duration
	Debug        bool
}

// Server wraps the HTTP server and router.
type Server struct {
	config Config
	http   *http.Server
	router *gin.Engine
}

// New creates a fully configured Server.
func New(cfg Config) *Server {
	if !cfg.Debug {
		gin.SetMode(gin.ReleaseMode)
	}

	r := gin.New()
	r.Use(gin.Logger())
	r.Use(gin.Recovery())
	r.Use(middleware.CORS())
	r.Use(middleware.RequestID())

	s := &Server{
		config: cfg,
		router: r,
	}
	s.registerRoutes()

	s.http = &http.Server{
		Addr:         fmt.Sprintf(":%s", cfg.Port),
		Handler:      r,
		ReadTimeout:  cfg.ReadTimeout,
		WriteTimeout: cfg.WriteTimeout,
		IdleTimeout:  cfg.IdleTimeout,
	}

	return s
}

// registerRoutes wires all API routes.
func (s *Server) registerRoutes() {
	r := s.router

	// Health
	r.GET("/health", handlers.Health)
	r.GET("/ready",  handlers.Ready)

	// API v1
	v1 := r.Group("/api/v1")
	{
		// Auth
		auth := v1.Group("/auth")
		auth.POST("/register", handlers.Register)
		auth.POST("/login",    handlers.Login)

		// Authenticated routes
		protected := v1.Group("/")
		protected.Use(middleware.Auth())
		{
			// Users
			protected.GET("/users/me",           handlers.GetCurrentUser)
			protected.PUT("/users/me",            handlers.UpdateCurrentUser)

			// Workspaces
			protected.GET("/workspace",           handlers.GetWorkspace)

			// Portfolios
			portfolios := protected.Group("/portfolios")
			portfolios.GET("/",          handlers.ListPortfolios)
			portfolios.POST("/",         handlers.CreatePortfolio)
			portfolios.GET("/:id",       handlers.GetPortfolio)
			portfolios.PUT("/:id",       handlers.UpdatePortfolio)
			portfolios.DELETE("/:id",    handlers.DeletePortfolio)

			// Portfolio items
			items := portfolios.Group("/:portfolio_id/items")
			items.GET("/",          handlers.ListPortfolioItems)
			items.POST("/",         handlers.CreatePortfolioItem)
			items.GET("/:id",       handlers.GetPortfolioItem)
			items.PUT("/:id",       handlers.UpdatePortfolioItem)
			items.DELETE("/:id",    handlers.DeletePortfolioItem)

			// WBS
			wbs := protected.Group("/wbs")
			wbs.POST("/",            handlers.CreateWBS)
			wbs.GET("/:id",          handlers.GetWBS)
			wbs.POST("/:id/stories", handlers.CreateStory)
			wbs.GET("/:id/stories",  handlers.ListStories)
			wbs.PUT("/:id/stories/:story_id", handlers.UpdateStory)

			// Projects
			projects := protected.Group("/projects")
			projects.GET("/",        handlers.ListProjects)
			projects.POST("/",       handlers.CreateProject)
			projects.GET("/:id",     handlers.GetProject)
			projects.PUT("/:id",     handlers.UpdateProject)

			// Events (SSE stream)
			protected.GET("/events/stream", handlers.EventStream)
		}
	}
}

// Start begins serving HTTP requests.
func (s *Server) Start() error {
	return s.http.ListenAndServe()
}

// Shutdown gracefully stops the server.
func (s *Server) Shutdown(ctx context.Context) error {
	return s.http.Shutdown(ctx)
}
