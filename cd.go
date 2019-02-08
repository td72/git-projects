package main

import (
	"github.com/urfave/cli"
	"errors"
	"strings"
	"fmt"
	"os/exec"
	"path/filepath"
	finder "github.com/b4b4r07/go-finder"
)

func list_projects(all bool) (finder.Items, error) {
	out, err := exec.Command("ghq", "list").Output()
	if err != nil {
		return finder.NewItems(), err
	}
	projects := strings.Split(string(out), "\n")
	items := finder.NewItems()
	for _, t := range GIT_PROJECTS_TARGETS {
		for _, p := range projects {
			if strings.Contains(p, t) || all {
				items.Add(p, p)
			}
		}
	}
	return items, nil
}

func choice_project(c *cli.Context) error {
	items, err := list_projects(c.NArg() > 0)
	fzf, err := finder.New("fzf", "--reverse", "--height", "20", "--prompt", `"cd >"`)
	selectedItems, err := fzf.Select(items)
	if err != nil {
		return err
	}
	if len(selectedItems) == 0 {
		return errors.New("no items")
	}
	fmt.Println(filepath.Join(GHQ_ROOT, selectedItems[0].(string)))
	return nil
}

func init() {
	command := cli.Command{
		Name:    "choice",
		Aliases: []string{"c"},
		Usage:   "choose project from list",
		Action:  choice_project,
	}
	App.Commands = append(App.Commands, command)
}
