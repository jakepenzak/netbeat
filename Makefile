.PHONY: install-hooks uninstall-hooks reinstall-hooks run

install-hooks:
	@echo "⏬ Installing pre-commit hooks..."
	@mkdir -p .git/hooks
	@for file in .hooks/*; do \
		if [ -f "$$file" ]; then \
			cp $$file .git/hooks/$$(basename $$file); \
			chmod +x .git/hooks/$$(basename $$file); \
		fi; \
	done
	@echo "✅ Hooks installed successfully!"

uninstall-hooks:
	@echo "Uninstalling pre-commit hooks..."
	@for file in .hooks/*; do \
		if [ -f "$$file" ]; then \
			rm -f .git/hooks/$$(basename $$file); \
		fi; \
	done
	@echo "✅ Hooks uninstalled successfully!"

reinstall-hooks: uninstall-hooks install-hooks

run:
	@if [ -z "$(HOOK)" ]; then \
		echo "Error: Please specify a hook file name using HOOK=<filename>"; \
		echo "Usage: make run HOOK=<filename>"; \
		exit 1; \
	fi
	@if [ ! -f ".hooks/$(HOOK)" ]; then \
		echo "Error: Hook file '.hooks/$(HOOK)' not found"; \
		exit 1; \
	fi
	@chmod +x .hooks/$(HOOK)
	@.hooks/$(HOOK)
