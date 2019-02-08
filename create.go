package main

import (
	"os/exec"
	"bufio"
	"os"
	"path/filepath"
	"strings"
	"errors"
	"fmt"
	"github.com/urfave/cli"
)

func write_readme(repo string) error {
	file, err := os.OpenFile(
		"./README.md",
		os.O_WRONLY|os.O_CREATE,
		0644,
	)
	if err != nil {
		return err
	}
	defer file.Close()
	fmt.Fprintf(file, "# %s", repo)
	return nil
}

func exec_with_stdout(c string, cmds ...string) error {
	cmd := exec.Command(c, cmds...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func create_project(_ *cli.Context) error {
	user := GITHUB_USER_NAME

	fmt.Printf(
		"input repository name [default: %s/]: ",
		GITHUB_USER_NAME,
	)
	stdin := bufio.NewScanner(os.Stdin)
	stdin.Scan()
	text := stdin.Text()
	var repo string
	if len(text) > 0 {
		arr := strings.Split(text, "/")
		if len(arr) == 2 {
			user = arr[0]
			repo = arr[1]
		} else if len(arr) == 1 {
			repo = text
		} else {
			return errors.New("too many arguments")
		}
	}
	path := filepath.Join(GHQ_ROOT, "github.com", user, repo)
	err := os.MkdirAll(path, 0755)
	if err != nil {
		return err
	}
	os.Chdir(path)
	err = exec.Command("git", "init").Run()
	if err != nil {
		return err
	}

	write_readme(repo)
	err = exec.Command("git", "add", ".").Run()
	if err != nil {
		return err
	}

	err = exec.Command(
		"git", "commit", "-m", `"initial commit"`,
	).Run()
	if err != nil {
		return err
	}

	if exec_with_stdout("hub", "create") != nil {
		return err
	}

	if exec_with_stdout("git", "push", "origin", "master") != nil {
		return err
	}

	return nil
}


func init() {
	command := cli.Command{
		Name:    "create",
		Usage:   "create new project [require: hub]",
		Action:  create_project,
	}
	App.Commands = append(App.Commands, command)
}
