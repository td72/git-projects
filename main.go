package main

import (
	"bufio"
	"fmt"
	"github.com/urfave/cli"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

const (
	GIT_PROJECTS_TARGETS_ENV = "GIT_PROJECTS_TARGETS"
)

var (
	App                  = cli.NewApp()
	GIT_PROJECTS_TARGETS []string
	GHQ_ROOT             string
	GITHUB_USER_NAME     string
)

func set_github_user_name() (err error) {
	xdg, ok := os.LookupEnv("XDG_CONFIG_HOME")
	if !ok {
		xdg = filepath.Join(os.Getenv("HOME"), ".config")
	}
	file, err := os.Open(filepath.Join(xdg, "hub"))
	if err != nil {
		GITHUB_USER_NAME = os.Getenv("USER")
		return
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		row := scanner.Text()
		if strings.Contains(row, "user: ") {
			GITHUB_USER_NAME = strings.Split(row, "user: ")[1]
			return
		}
	}
	if err = scanner.Err(); err != nil {
		return
	}
	return nil
}

func initialize(_ *cli.Context) error {
	targets, _ := os.LookupEnv(GIT_PROJECTS_TARGETS_ENV)
	for _, t := range strings.Split(targets, ":") {
		GIT_PROJECTS_TARGETS = append(GIT_PROJECTS_TARGETS, t)
	}

	_, err := exec.LookPath("ghq")
	if err != nil {
		return fmt.Errorf("ghq command not found.\nplease install ghq.")
	}

	out, err := exec.Command("ghq", "root").Output()
	if err != nil {
		return err
	}
	GHQ_ROOT = strings.Trim(string(out), "\n")

	set_github_user_name()
	return nil
}

func main() {
	App.Name = "git-projects"
	App.Usage = "git projects"
	App.Version = "0.0.1"
	App.Before = initialize
	App.Setup()
	App.Run(os.Args)
}
