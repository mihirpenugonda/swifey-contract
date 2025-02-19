import { Idl } from "@coral-xyz/anchor";

export type Swifey = {
  "version": "0.1.0",
  "name": "swifey",
  "instructions": [
    {
      "name": "configure",
      "accounts": [
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "globalConfig",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "newConfig",
          "type": {
            "defined": "states::Config"
          }
        }
      ]
    },
    {
      "name": "launch",
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "globalConfig",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "bondingCurve",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "curveTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenMetadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "metadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "symbol",
          "type": "string"
        },
        {
          "name": "uri",
          "type": "string"
        }
      ]
    },
    {
      "name": "swap",
      "accounts": [
        {
          "name": "user",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "globalConfig",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "feeRecipient",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondingCurve",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "curveTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "direction",
          "type": "u8"
        },
        {
          "name": "minOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "migrate",
      "accounts": [
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "config",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondingCurve",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "curveTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "curveSolAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "raydiumPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "feeRecipient",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "instructions::MigrateParams"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "bondingCurve",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "virtualTokenReserve",
            "type": "u64"
          },
          {
            "name": "virtualSolReserve",
            "type": "u64"
          },
          {
            "name": "realTokenReserve",
            "type": "u64"
          },
          {
            "name": "realSolReserve",
            "type": "u64"
          },
          {
            "name": "tokenTotalSupply",
            "type": "u64"
          },
          {
            "name": "isCompleted",
            "type": "bool"
          },
          {
            "name": "isMigrated",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "config",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "publicKey"
          },
          {
            "name": "feeRecipient",
            "type": "publicKey"
          },
          {
            "name": "curveLimit",
            "type": "u64"
          },
          {
            "name": "initialVirtualTokenReserve",
            "type": "u64"
          },
          {
            "name": "initialVirtualSolReserve",
            "type": "u64"
          },
          {
            "name": "initialRealTokenReserve",
            "type": "u64"
          },
          {
            "name": "totalTokenSupply",
            "type": "u64"
          },
          {
            "name": "buyFeePercentage",
            "type": "f64"
          },
          {
            "name": "sellFeePercentage",
            "type": "f64"
          },
          {
            "name": "migrationFeePercentage",
            "type": "f64"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "MigrateParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "minimumSolAmount",
            "type": "u64"
          },
          {
            "name": "minimumTokenAmount",
            "type": "u64"
          }
        ]
      }
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "UnauthorizedAddress",
      "msg": "Unauthorized address"
    },
    {
      "code": 6001,
      "name": "CurveLimitReached",
      "msg": "Curve limit reached"
    },
    {
      "code": 6002,
      "name": "IncorrectValueRange",
      "msg": "Value is not in expected range"
    },
    {
      "code": 6003,
      "name": "InsufficientAmountOut",
      "msg": "Amount out is smaller than required amount"
    },
    {
      "code": 6004,
      "name": "InsufficientFunds",
      "msg": "Insufficient funds"
    },
    {
      "code": 6005,
      "name": "IncorrectFeeRecipient",
      "msg": "Incorrect fee recipient"
    },
    {
      "code": 6006,
      "name": "InvalidReserves",
      "msg": "An overflow or underflow occurred during calculation"
    },
    {
      "code": 6007,
      "name": "CurveNotInitialized",
      "msg": "Curve is not initialized"
    },
    {
      "code": 6008,
      "name": "CurveNotCompleted",
      "msg": "Curve is not completed"
    },
    {
      "code": 6009,
      "name": "AlreadyMigrated",
      "msg": "Already migrated to Raydium"
    },
    {
      "code": 6010,
      "name": "MathOverflow",
      "msg": "Mathematical operation overflow"
    },
    {
      "code": 6011,
      "name": "InsufficientSolBalance",
      "msg": "Insufficient SOL balance"
    },
    {
      "code": 6012,
      "name": "InsufficientTokenBalance",
      "msg": "Insufficient token balance"
    },
    {
      "code": 6013,
      "name": "InvalidPoolOwner",
      "msg": "Invalid pool owner"
    },
    {
      "code": 6014,
      "name": "InvalidPoolState",
      "msg": "Invalid pool state"
    },
    {
      "code": 6015,
      "name": "InvalidPoolTokens",
      "msg": "Invalid pool tokens"
    },
    {
      "code": 6016,
      "name": "SlippageExceeded",
      "msg": "Slippage tolerance exceeded"
    }
  ]
};

export const IDL = {
  "version": "0.1.0",
  "name": "swifey",
  "instructions": [
    {
      "name": "configure",
      "accounts": [
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "globalConfig",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "newConfig",
          "type": {
            "defined": "states::Config"
          }
        }
      ]
    },
    {
      "name": "launch",
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "globalConfig",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "bondingCurve",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "curveTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenMetadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "metadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "symbol",
          "type": "string"
        },
        {
          "name": "uri",
          "type": "string"
        }
      ]
    },
    {
      "name": "swap",
      "accounts": [
        {
          "name": "user",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "globalConfig",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "feeRecipient",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondingCurve",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "curveTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "direction",
          "type": "u8"
        },
        {
          "name": "minOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "migrate",
      "accounts": [
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "config",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondingCurve",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "curveTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "curveSolAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "raydiumPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "feeRecipient",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "instructions::MigrateParams"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "bondingCurve",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "virtualTokenReserve",
            "type": "u64"
          },
          {
            "name": "virtualSolReserve",
            "type": "u64"
          },
          {
            "name": "realTokenReserve",
            "type": "u64"
          },
          {
            "name": "realSolReserve",
            "type": "u64"
          },
          {
            "name": "tokenTotalSupply",
            "type": "u64"
          },
          {
            "name": "isCompleted",
            "type": "bool"
          },
          {
            "name": "isMigrated",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "config",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "publicKey"
          },
          {
            "name": "feeRecipient",
            "type": "publicKey"
          },
          {
            "name": "curveLimit",
            "type": "u64"
          },
          {
            "name": "initialVirtualTokenReserve",
            "type": "u64"
          },
          {
            "name": "initialVirtualSolReserve",
            "type": "u64"
          },
          {
            "name": "initialRealTokenReserve",
            "type": "u64"
          },
          {
            "name": "totalTokenSupply",
            "type": "u64"
          },
          {
            "name": "buyFeePercentage",
            "type": "f64"
          },
          {
            "name": "sellFeePercentage",
            "type": "f64"
          },
          {
            "name": "migrationFeePercentage",
            "type": "f64"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "MigrateParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "minimumSolAmount",
            "type": "u64"
          },
          {
            "name": "minimumTokenAmount",
            "type": "u64"
          }
        ]
      }
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "UnauthorizedAddress",
      "msg": "Unauthorized address"
    },
    {
      "code": 6001,
      "name": "CurveLimitReached",
      "msg": "Curve limit reached"
    },
    {
      "code": 6002,
      "name": "IncorrectValueRange",
      "msg": "Value is not in expected range"
    },
    {
      "code": 6003,
      "name": "InsufficientAmountOut",
      "msg": "Amount out is smaller than required amount"
    },
    {
      "code": 6004,
      "name": "InsufficientFunds",
      "msg": "Insufficient funds"
    },
    {
      "code": 6005,
      "name": "IncorrectFeeRecipient",
      "msg": "Incorrect fee recipient"
    },
    {
      "code": 6006,
      "name": "InvalidReserves",
      "msg": "An overflow or underflow occurred during calculation"
    },
    {
      "code": 6007,
      "name": "CurveNotInitialized",
      "msg": "Curve is not initialized"
    },
    {
      "code": 6008,
      "name": "CurveNotCompleted",
      "msg": "Curve is not completed"
    },
    {
      "code": 6009,
      "name": "AlreadyMigrated",
      "msg": "Already migrated to Raydium"
    },
    {
      "code": 6010,
      "name": "MathOverflow",
      "msg": "Mathematical operation overflow"
    },
    {
      "code": 6011,
      "name": "InsufficientSolBalance",
      "msg": "Insufficient SOL balance"
    },
    {
      "code": 6012,
      "name": "InsufficientTokenBalance",
      "msg": "Insufficient token balance"
    },
    {
      "code": 6013,
      "name": "InvalidPoolOwner",
      "msg": "Invalid pool owner"
    },
    {
      "code": 6014,
      "name": "InvalidPoolState",
      "msg": "Invalid pool state"
    },
    {
      "code": 6015,
      "name": "InvalidPoolTokens",
      "msg": "Invalid pool tokens"
    },
    {
      "code": 6016,
      "name": "SlippageExceeded",
      "msg": "Slippage tolerance exceeded"
    }
  ]
} as Idl;