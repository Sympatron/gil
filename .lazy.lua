return {
	{
		"mrcjkb/rustaceanvim",
		config = function()
			vim.g.rustaceanvim = {
				server = {
					default_settings = {
						-- rust-analyzer language server configuration
						["rust-analyzer"] = {
							cargo = {
								allFeatures = false,
								noDefaultFeatures = true,
								features = { "async" },
							},
							checkOnSave = {
								allFeatures = false,
								noDefaultFeatures = true,
								features = { "async" },
							},
						},
					},
				},
			}
		end,
	},
}
