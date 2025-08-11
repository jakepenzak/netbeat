.PHONY: install-hooks uninstall-hooks reinstall-hooks

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
