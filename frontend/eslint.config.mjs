// @ts-check
import { antfu } from "@antfu/eslint-config";
import format from "eslint-plugin-format";
import tailwind from "eslint-plugin-tailwindcss";

export default antfu({
	vue: true,
	typescript: true,
	formatters: {
		css: true,
		svg: true,
		html: true,
	},
})
	.override("antfu/vue/rules", {
		rules: {
			"vue/no-v-html": "error",
			"vue/max-attributes-per-line": ["error", { singleline: 5, multiline: 1 }],
		},
	})
	.removeRules("vue/multi-word-component-names")
	.override("tailwindcss:rules", {
		rules: {
			"tailwindcss/no-custom-classname": [
				"error",
				{
					whitelist: ["header-anchor", "custom-block-title", "content"],
				},
			],
		},
	})
	.append(...tailwind.configs["flat/recommended"], {
		files: ["**/*.css", "**/*.scss"],
		languageOptions: {
			parser: format.parserPlain,
		},
		plugins: {
			format,
		},
		rules: {
			"format/prettier": ["error", { parser: "css", tabWidth: 2 }],
		},
	});
