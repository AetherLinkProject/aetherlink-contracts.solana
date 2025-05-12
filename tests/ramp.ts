import * as anchor from '@coral-xyz/anchor';
import { Program, BN } from '@coral-xyz/anchor';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { assert } from 'chai';
import { Ramp } from '../target/types/ramp.js';
import * as secp256k1 from 'secp256k1';


describe('ramp', () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.Ramp as Program<Ramp>;
    const admin = provider.wallet as anchor.Wallet;
    const bridge = Keypair.generate();
    const user = Keypair.generate();
    let rampConfigPda: PublicKey;

    console.log(`   admin's public key: ${admin.publicKey}`);
    console.log(`   admin's public key: ${bridge.publicKey}`);
    console.log(`   admin's public key: ${user.publicKey}`);

    before(async () => {
        // Airdrop 2 SOL to the test wallet before running tests
        await provider.connection.requestAirdrop(admin.publicKey, 2e9);
        await provider.connection.requestAirdrop(bridge.publicKey, 2e9);
        await provider.connection.requestAirdrop(user.publicKey, 2e9);

        [rampConfigPda] = await PublicKey.findProgramAddressSync(
            [Buffer.from('ramp_config')],
            program.programId
        );
        console.log(`   before start rampConfigPda:${rampConfigPda}`);
        console.log(`   loaded programId from workspace: ${program.programId.toBase58()}`);
    });

    it('Initialize ramp config', async () => {
        const preAccount = await provider.connection.getAccountInfo(rampConfigPda);
        if (preAccount) {
            console.log('PDA already exists, treat as initialized.');
            return;
        }
        await program.methods
            .initialize(admin.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                user: admin.publicKey,
                systemProgram: SystemProgram.programId,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        const rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        assert.isTrue(rampConfig.isInitialized, 'is_initialized should be true');
        assert.equal(rampConfig.authority.toBase58(), admin.publicKey.toBase58(), 'authority should match');
    });

    it('Initialize ramp config: should fail if already initialized', async function () {
        this.timeout(10000);
        let threw = false;
        try {
            await program.methods
                .initialize(admin.publicKey)
                .accounts({
                    rampConfig: rampConfigPda,
                    user: admin.publicKey,
                    systemProgram: SystemProgram.programId,
                } as any)
                .signers([provider.wallet.payer])
                .rpc();
        } catch (e: any) {
            threw = true;
            // Debug print error for analysis
            console.log('AlreadyInitialized error:', e.message, e.error);
            // AnchorError: e.error.errorCode or e.error.errorName
            // Sometimes Anchor does not parse errorName, so check errorCode (6000) or message contains 0x1770
            // In practice, system_program will throw 'account already in use' (0x0) before Anchor's AlreadyInitialized
            if (e.error && e.error.errorCode !== undefined) {
                assert.equal(e.error.errorCode, 6000, 'Should throw AlreadyInitialized (code 6000)');
            } else if (e.message) {
                if (e.message.includes('already in use') || e.message.includes('custom program error: 0x0')) {
                    assert.isTrue(true, 'System program throws account already in use');
                } else {
                    assert.fail('Error did not contain AlreadyInitialized or account already in use');
                }
            } else {
                assert.fail('Error did not contain AlreadyInitialized');
            }
        }
        assert.isTrue(threw, 'Should throw if already initialized');
    });

    it('Set admin', async () => {
        // Generate a new authority
        const newAuthority = Keypair.generate();
        await program.methods
            .setAdmin(newAuthority.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                currentAuthority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        const rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        assert.equal(rampConfig.authority.toBase58(), newAuthority.publicKey.toBase58(), 'authority should be updated');
        // Restore admin as authority for subsequent tests
        await program.methods
            .setAdmin(admin.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                currentAuthority: newAuthority.publicKey,
            } as any)
            .signers([newAuthority])
            .rpc();
    });

    it('Set admin: should fail if not current authority', async () => {
        // Use a random keypair as unauthorized
        const notAdmin = Keypair.generate();
        let threw = false;
        try {
            await program.methods
                .setAdmin(Keypair.generate().publicKey)
                .accounts({
                    rampConfig: rampConfigPda,
                    currentAuthority: notAdmin.publicKey,
                } as any)
                .signers([notAdmin])
                .rpc();
        } catch (e: any) {
            threw = true;
            assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
        }
        assert.isTrue(threw, 'Should throw if not current authority');
    });

    it('Set oracle nodes', async () => {
        // Generate 8 new oracle node pubkeys
        const nodePubkeys = Array.from({ length: 8 }, () => Keypair.generate().publicKey);
        await program.methods
            .setOracleNodes(nodePubkeys)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        const rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        assert.deepEqual(rampConfig.oracleNodes.map((pk: any) => pk.toBase58()), nodePubkeys.map(pk => pk.toBase58()), 'oracleNodes should match');
    });

    it('Set oracle nodes: should fail if not authority', async () => {
        const notAdmin = Keypair.generate();
        const nodePubkeys = Array.from({ length: 8 }, () => Keypair.generate().publicKey);
        let threw = false;
        try {
            await program.methods
                .setOracleNodes(nodePubkeys)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: notAdmin.publicKey,
                } as any)
                .signers([notAdmin])
                .rpc();
        } catch (e: any) {
            threw = true;
            assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
        }
        assert.isTrue(threw, 'Should throw if not authority');
    });

    it('Add sender', async () => {
        // Clean up senderWhitelist before test
        let rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        for (const pk of rampConfig.senderWhitelist) {
            if (!pk.equals(PublicKey.default)) {
                try {
                    await program.methods
                        .removeSender(pk)
                        .accounts({
                            rampConfig: rampConfigPda,
                            authority: admin.publicKey,
                        } as any)
                        .signers([provider.wallet.payer])
                        .rpc();
                } catch (e) {}
            }
        }
        const sender = Keypair.generate();
        await program.methods
            .addSender(sender.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        assert.include(rampConfig.senderWhitelist.map((pk: any) => pk.toBase58()), sender.publicKey.toBase58(), 'sender should be whitelisted');
    });

    it('Remove sender', async () => {
        // Clean up senderWhitelist before test
        let rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        for (const pk of rampConfig.senderWhitelist) {
            if (!pk.equals(PublicKey.default)) {
                try {
                    await program.methods
                        .removeSender(pk)
                        .accounts({
                            rampConfig: rampConfigPda,
                            authority: admin.publicKey,
                        } as any)
                        .signers([provider.wallet.payer])
                        .rpc();
                } catch (e) {}
            }
        }
        const sender = Keypair.generate();
        // Add sender first
        await program.methods
            .addSender(sender.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Remove sender
        await program.methods
            .removeSender(sender.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        assert.notInclude(rampConfig.senderWhitelist.map((pk: any) => pk.toBase58()), sender.publicKey.toBase58(), 'sender should be removed from whitelist');
    });

    it('Set chain whitelist', async () => {
        const sourceChainIds = [1, 2, 3, 4, 5, 6, 7, 8].map(x => new BN(x));
        const destinationChainIds = [11, 12, 13, 14, 15, 16, 17, 18].map(x => new BN(x));
        await program.methods
            .setChainWhitelist(sourceChainIds, destinationChainIds)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        const rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        assert.deepEqual(rampConfig.sourceChainWhitelist.map((bn: BN) => bn.toNumber()), sourceChainIds.map((bn: BN) => bn.toNumber()), 'sourceChainWhitelist should match');
        assert.deepEqual(rampConfig.destinationChainWhitelist.map((bn: BN) => bn.toNumber()), destinationChainIds.map((bn: BN) => bn.toNumber()), 'destinationChainWhitelist should match');
    });

    it('Send request', async () => {
        // Clean up senderWhitelist before test
        let rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        for (const pk of rampConfig.senderWhitelist) {
            if (!pk.equals(PublicKey.default)) {
                try {
                    await program.methods
                        .removeSender(pk)
                        .accounts({
                            rampConfig: rampConfigPda,
                            authority: admin.publicKey,
                        } as any)
                        .signers([provider.wallet.payer])
                        .rpc();
                } catch (e) {}
            }
        }
        // Prepare a sender and add to whitelist
        const sender = Keypair.generate();
        await program.methods
            .addSender(sender.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Prepare a destination chain id and add to whitelist
        const dstChainId = new BN(1001);
        const rampConfigBefore = await program.account.rampConfig.fetch(rampConfigPda);
        const newDstWhitelist = rampConfigBefore.destinationChainWhitelist.map((bn: BN) => bn.toNumber());
        newDstWhitelist[0] = dstChainId.toNumber();
        await program.methods
            .setChainWhitelist(rampConfigBefore.sourceChainWhitelist, newDstWhitelist.map(x => new BN(x)))
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Prepare receiver, message, token_transfer_metadata
        const receiver = '0xReceiverAddress';
        const message = Buffer.from('hello world');
        const symbolBuffer = Buffer.from('USDC'.padEnd(16, '\0'));
        const tokenTransferMetadata = {
            targetChainId: new BN(2002),
            tokenAddress: sender.publicKey,
            symbol: Array.from(symbolBuffer),
            amount: new BN(123456),
            extraData: 'extra',
        };
        // Call send_request (should not throw)
        await program.methods
            .sendRequest(dstChainId, receiver, message, tokenTransferMetadata)
            .accounts({
                rampConfig: rampConfigPda,
                sender: sender.publicKey,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            } as any)
            .signers([sender])
            .rpc();
        // Unauthorized sender should fail
        const notWhitelisted = Keypair.generate();
        let threw = false;
        try {
            await program.methods
                .sendRequest(dstChainId, receiver, message, tokenTransferMetadata)
                .accounts({
                    rampConfig: rampConfigPda,
                    sender: notWhitelisted.publicKey,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                } as any)
                .signers([notWhitelisted])
                .rpc();
        } catch (e) {
            threw = true;
        }
        assert.isTrue(threw, 'Unauthorized sender should throw');
        // Unauthorized chain id should fail
        const notWhitelistedChainId = new BN(9999);
        threw = false;
        try {
            await program.methods
                .sendRequest(notWhitelistedChainId, receiver, message, tokenTransferMetadata)
                .accounts({
                    rampConfig: rampConfigPda,
                    sender: sender.publicKey,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                } as any)
                .signers([sender])
                .rpc();
        } catch (e) {
            threw = true;
        }
        assert.isTrue(threw, 'Unauthorized chain id should throw');
    });

    it('Add sender: should fail if whitelist is full', async function () {
        this.timeout(10000);
        // Fill the whitelist
        let rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        // Clean up first
        for (const pk of rampConfig.senderWhitelist) {
            if (!pk.equals(PublicKey.default)) {
                try {
                    await program.methods
                        .removeSender(pk)
                        .accounts({
                            rampConfig: rampConfigPda,
                            authority: admin.publicKey,
                        } as any)
                        .signers([provider.wallet.payer])
                        .rpc();
                } catch (e) {}
            }
        }
        // Fill
        for (let i = 0; i < 8; i++) {
            const sender = Keypair.generate();
            await program.methods
                .addSender(sender.publicKey)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: admin.publicKey,
                } as any)
                .signers([provider.wallet.payer])
                .rpc();
        }
        // Try to add one more
        const extraSender = Keypair.generate();
        let threw = false;
        try {
            await program.methods
                .addSender(extraSender.publicKey)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: admin.publicKey,
                } as any)
                .signers([provider.wallet.payer])
                .rpc();
        } catch (e: any) {
            threw = true;
            if (e.error && e.error.errorName) {
                assert.equal(e.error.errorName, 'SenderWhitelistFull', 'Should throw SenderWhitelistFull');
            } else if (e.message) {
                assert.include(e.message, 'SenderWhitelistFull', 'Should throw SenderWhitelistFull');
            } else {
                assert.fail('Error did not contain SenderWhitelistFull');
            }
        }
        assert.isTrue(threw, 'Should throw if whitelist is full');
        // Clean up for other tests
        rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        for (const pk of rampConfig.senderWhitelist) {
            if (!pk.equals(PublicKey.default)) {
                try {
                    await program.methods
                        .removeSender(pk)
                        .accounts({
                            rampConfig: rampConfigPda,
                            authority: admin.publicKey,
                        } as any)
                        .signers([provider.wallet.payer])
                        .rpc();
                } catch (e) {}
            }
        }
    });

    it('Add sender: should fail if not authority', async () => {
        const sender = Keypair.generate();
        const notAdmin = Keypair.generate();
        let threw = false;
        try {
            await program.methods
                .addSender(sender.publicKey)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: notAdmin.publicKey,
                } as any)
                .signers([notAdmin])
                .rpc();
        } catch (e: any) {
            threw = true;
            assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
        }
        assert.isTrue(threw, 'Should throw if not authority');
    });

    it('Remove sender: should fail if sender not found', async () => {
        const notWhitelisted = Keypair.generate();
        let threw = false;
        try {
            await program.methods
                .removeSender(notWhitelisted.publicKey)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: admin.publicKey,
                } as any)
                .signers([provider.wallet.payer])
                .rpc();
        } catch (e: any) {
            threw = true;
            assert.include(e.message, 'SenderNotFound', 'Should throw SenderNotFound');
        }
        assert.isTrue(threw, 'Should throw if sender not found');
    });

    it('Remove sender: should fail if not authority', async () => {
        // Add a sender first
        const sender = Keypair.generate();
        await program.methods
            .addSender(sender.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        const notAdmin = Keypair.generate();
        let threw = false;
        try {
            await program.methods
                .removeSender(sender.publicKey)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: notAdmin.publicKey,
                } as any)
                .signers([notAdmin])
                .rpc();
        } catch (e: any) {
            threw = true;
            assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
        }
        assert.isTrue(threw, 'Should throw if not authority');
        // Clean up
        await program.methods
            .removeSender(sender.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
    });

    it('Set chain whitelist: should fail if not authority', async () => {
        const notAdmin = Keypair.generate();
        const sourceChainIds = [1, 2, 3, 4, 5, 6, 7, 8].map(x => new BN(x));
        const destinationChainIds = [11, 12, 13, 14, 15, 16, 17, 18].map(x => new BN(x));
        let threw = false;
        try {
            await program.methods
                .setChainWhitelist(sourceChainIds, destinationChainIds)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: notAdmin.publicKey,
                } as any)
                .signers([notAdmin])
                .rpc();
        } catch (e: any) {
            threw = true;
            assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
        }
        assert.isTrue(threw, 'Should throw if not authority');
    });

    it('Send request: should fail if sender not whitelisted', async function () {
        this.timeout(10000);
        // Clean up senderWhitelist before test
        let rampConfig = await program.account.rampConfig.fetch(rampConfigPda);
        for (const pk of rampConfig.senderWhitelist) {
            if (!pk.equals(PublicKey.default)) {
                try {
                    await program.methods
                        .removeSender(pk)
                        .accounts({
                            rampConfig: rampConfigPda,
                            authority: admin.publicKey,
                        } as any)
                        .signers([provider.wallet.payer])
                        .rpc();
                } catch (e) {}
            }
        }
        // Prepare a sender not in whitelist
        const notWhitelisted = Keypair.generate();
        const dstChainId = new BN(1001);
        const receiver = '0xReceiverAddress';
        const message = Buffer.from('hello world');
        const symbolBuffer = Buffer.from('USDC'.padEnd(16, '\0'));
        const tokenTransferMetadata = {
            targetChainId: new BN(2002),
            tokenAddress: notWhitelisted.publicKey,
            symbol: Array.from(symbolBuffer),
            amount: new BN(123456),
            extraData: 'extra',
        };
        let threw = false;
        try {
            await program.methods
                .sendRequest(dstChainId, receiver, message, tokenTransferMetadata)
                .accounts({
                    rampConfig: rampConfigPda,
                    sender: notWhitelisted.publicKey,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                } as any)
                .signers([notWhitelisted])
                .rpc();
        } catch (e: any) {
            threw = true;
            if (e.error && e.error.errorName) {
                assert.equal(e.error.errorName, 'Unauthorized', 'Should throw Unauthorized');
            } else if (e.message) {
                assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
            } else {
                assert.fail('Error did not contain Unauthorized');
            }
        }
        assert.isTrue(threw, 'Should throw if sender not whitelisted');
    });

    it('Send request: should fail if destination chain not whitelisted', async () => {
        // Add a sender to whitelist
        const sender = Keypair.generate();
        await program.methods
            .addSender(sender.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Use a chain id not in whitelist
        const notWhitelistedChainId = new BN(9999);
        const receiver = '0xReceiverAddress';
        const message = Buffer.from('hello world');
        const symbolBuffer = Buffer.from('USDC'.padEnd(16, '\0'));
        const tokenTransferMetadata = {
            targetChainId: new BN(2002),
            tokenAddress: sender.publicKey,
            symbol: Array.from(symbolBuffer),
            amount: new BN(123456),
            extraData: 'extra',
        };
        let threw = false;
        try {
            await program.methods
                .sendRequest(notWhitelistedChainId, receiver, message, tokenTransferMetadata)
                .accounts({
                    rampConfig: rampConfigPda,
                    sender: sender.publicKey,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                } as any)
                .signers([sender])
                .rpc();
        } catch (e: any) {
            threw = true;
            assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
        }
        assert.isTrue(threw, 'Should throw if destination chain not whitelisted');
        // Clean up
        await program.methods
            .removeSender(sender.publicKey)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
    });

    it('Transmit: should succeed with valid signatures and threshold', async function () {
        this.timeout(10000);
        // Set up 2 oracle nodes with real secp256k1 keypairs
        const priv1 = Buffer.alloc(32, 1); // deterministic for test
        const priv2 = Buffer.alloc(32, 2);
        const pub1 = secp256k1.publicKeyCreate(priv1, false).slice(1); // 64 bytes
        const pub2 = secp256k1.publicKeyCreate(priv2, false).slice(1);
        // Convert to Solana Pubkey (32 bytes)
        const node1 = new PublicKey(pub1.slice(0, 32));
        const node2 = new PublicKey(pub2.slice(0, 32));
        await program.methods
            .setOracleNodes([node1, node2, ...Array(6).fill(PublicKey.default)])
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Prepare report_context, message, meta
        const reportContext = Buffer.from('report_context');
        const message = Buffer.from('msg');
        const meta = Buffer.from('meta');
        // Hash = keccak(reportContext || message || meta)
        const Keccak = require('keccak');
        const hasher = new Keccak('keccak256');
        hasher.update(reportContext);
        hasher.update(message);
        hasher.update(meta);
        const hash = hasher.digest(); // 32 bytes
        // Sign hash with priv1, priv2
        const sigObj1 = secp256k1.ecdsaSign(hash, priv1);
        const sigObj2 = secp256k1.ecdsaSign(hash, priv2);
        // 64-byte signature, 1-byte recid
        const sig1 = Buffer.from(sigObj1.signature);
        const recId1 = sigObj1.recid;
        const sig2 = Buffer.from(sigObj2.signature);
        const recId2 = sigObj2.recid;
        // Assemble data: [rc_len|rc_bytes|msg_len|msg_bytes|meta_len|meta_bytes|sig_count|sig1|rec_id1|sig2|rec_id2]
        const data = Buffer.concat([
            Buffer.from(Uint32Array.of(reportContext.length).buffer),
            reportContext,
            Buffer.from(Uint32Array.of(message.length).buffer),
            message,
            Buffer.from(Uint32Array.of(meta.length).buffer),
            meta,
            Buffer.from(Uint32Array.of(2).buffer), // sig_count
            sig1,
            Buffer.from([recId1]),
            sig2,
            Buffer.from([recId2]),
        ]);
        // Call transmit (should not throw)
        await program.methods
            .transmit(data as any)
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey, // not used in transmit, but for workspace typing
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // If no error, test passes
    });

    it('Transmit: should fail if signatures are below threshold', async function () {
        this.timeout(10000);
        // Set up 2 oracle nodes
        const priv1 = Buffer.alloc(32, 1);
        const priv2 = Buffer.alloc(32, 2);
        const pub1 = secp256k1.publicKeyCreate(priv1, false).slice(1);
        const pub2 = secp256k1.publicKeyCreate(priv2, false).slice(1);
        const node1 = new PublicKey(pub1.slice(0, 32));
        const node2 = new PublicKey(pub2.slice(0, 32));
        await program.methods
            .setOracleNodes([node1, node2, ...Array(6).fill(PublicKey.default)])
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Prepare report_context, message, meta
        const reportContext = Buffer.from('report_context');
        const message = Buffer.from('msg');
        const meta = Buffer.from('meta');
        // Hash = keccak(reportContext || message || meta)
        const Keccak = require('keccak');
        const hasher = new Keccak('keccak256');
        hasher.update(reportContext);
        hasher.update(message);
        hasher.update(meta);
        const hash = hasher.digest();
        // Only 1 signature (threshold is 2)
        const sigObj1 = secp256k1.ecdsaSign(hash, priv1);
        const sig1 = Buffer.from(sigObj1.signature);
        const recId1 = sigObj1.recid;
        const data = Buffer.concat([
            Buffer.from(Uint32Array.of(reportContext.length).buffer),
            reportContext,
            Buffer.from(Uint32Array.of(message.length).buffer),
            message,
            Buffer.from(Uint32Array.of(meta.length).buffer),
            meta,
            Buffer.from(Uint32Array.of(1).buffer), // sig_count
            sig1,
            Buffer.from([recId1]),
        ]);
        let threw = false;
        try {
            await program.methods
                .transmit(data as any)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: admin.publicKey,
                } as any)
                .signers([provider.wallet.payer])
                .rpc();
        } catch (e: any) {
            threw = true;
            if (e.error && e.error.errorName) {
                assert.equal(e.error.errorName, 'Unauthorized', 'Should throw Unauthorized');
            } else if (e.message) {
                assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
            } else {
                assert.fail('Error did not contain Unauthorized');
            }
        }
        assert.isTrue(threw, 'Should throw if signatures are below threshold');
    });

    it('Transmit: should fail if signature is not from oracle node', async function () {
        this.timeout(10000);
        // Set up 2 oracle nodes
        const priv1 = Buffer.alloc(32, 1);
        const priv2 = Buffer.alloc(32, 2);
        const pub1 = secp256k1.publicKeyCreate(priv1, false).slice(1);
        const pub2 = secp256k1.publicKeyCreate(priv2, false).slice(1);
        const node1 = new PublicKey(pub1.slice(0, 32));
        const node2 = new PublicKey(pub2.slice(0, 32));
        await program.methods
            .setOracleNodes([node1, node2, ...Array(6).fill(PublicKey.default)])
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Prepare report_context, message, meta
        const reportContext = Buffer.from('report_context');
        const message = Buffer.from('msg');
        const meta = Buffer.from('meta');
        // Hash = keccak(reportContext || message || meta)
        const Keccak = require('keccak');
        const hasher = new Keccak('keccak256');
        hasher.update(reportContext);
        hasher.update(message);
        hasher.update(meta);
        const hash = hasher.digest();
        // Use a key not in oracle_nodes
        const priv3 = Buffer.alloc(32, 3);
        const sigObj3 = secp256k1.ecdsaSign(hash, priv3);
        const sig3 = Buffer.from(sigObj3.signature);
        const recId3 = sigObj3.recid;
        // 2 signatures, both from priv3 (not in oracle_nodes)
        const data = Buffer.concat([
            Buffer.from(Uint32Array.of(reportContext.length).buffer),
            reportContext,
            Buffer.from(Uint32Array.of(message.length).buffer),
            message,
            Buffer.from(Uint32Array.of(meta.length).buffer),
            meta,
            Buffer.from(Uint32Array.of(2).buffer), // sig_count
            sig3,
            Buffer.from([recId3]),
            sig3,
            Buffer.from([recId3]),
        ]);
        let threw = false;
        try {
            await program.methods
                .transmit(data as any)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: admin.publicKey,
                } as any)
                .signers([provider.wallet.payer])
                .rpc();
        } catch (e: any) {
            threw = true;
            if (e.error && e.error.errorName) {
                assert.equal(e.error.errorName, 'Unauthorized', 'Should throw Unauthorized');
            } else if (e.message) {
                assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
            } else {
                assert.fail('Error did not contain Unauthorized');
            }
        }
        assert.isTrue(threw, 'Should throw if signature is not from oracle node');
    });

    it('Transmit: should fail if duplicate signatures from same oracle node', async function () {
        this.timeout(10000);
        // Set up 2 oracle nodes
        const priv1 = Buffer.alloc(32, 1);
        const priv2 = Buffer.alloc(32, 2);
        const pub1 = secp256k1.publicKeyCreate(priv1, false).slice(1);
        const pub2 = secp256k1.publicKeyCreate(priv2, false).slice(1);
        const node1 = new PublicKey(pub1.slice(0, 32));
        const node2 = new PublicKey(pub2.slice(0, 32));
        await program.methods
            .setOracleNodes([node1, node2, ...Array(6).fill(PublicKey.default)])
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Prepare report_context, message, meta
        const reportContext = Buffer.from('report_context');
        const message = Buffer.from('msg');
        const meta = Buffer.from('meta');
        // Hash = keccak(reportContext || message || meta)
        const Keccak = require('keccak');
        const hasher = new Keccak('keccak256');
        hasher.update(reportContext);
        hasher.update(message);
        hasher.update(meta);
        const hash = hasher.digest();
        // Both signatures from priv1 (same node)
        const sigObj1 = secp256k1.ecdsaSign(hash, priv1);
        const sig1 = Buffer.from(sigObj1.signature);
        const recId1 = sigObj1.recid;
        // 2 signatures, both from priv1
        const data = Buffer.concat([
            Buffer.from(Uint32Array.of(reportContext.length).buffer),
            reportContext,
            Buffer.from(Uint32Array.of(message.length).buffer),
            message,
            Buffer.from(Uint32Array.of(meta.length).buffer),
            meta,
            Buffer.from(Uint32Array.of(2).buffer), // sig_count
            sig1,
            Buffer.from([recId1]),
            sig1,
            Buffer.from([recId1]),
        ]);
        let threw = false;
        try {
            await program.methods
                .transmit(data as any)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: admin.publicKey,
                } as any)
                .signers([provider.wallet.payer])
                .rpc();
        } catch (e: any) {
            threw = true;
            if (e.error && e.error.errorName) {
                assert.equal(e.error.errorName, 'Unauthorized', 'Should throw Unauthorized');
            } else if (e.message) {
                assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
            } else {
                assert.fail('Error did not contain Unauthorized');
            }
        }
        assert.isTrue(threw, 'Should throw if duplicate signatures from same oracle node');
    });

    it('Transmit: should fail if data is tampered (message changed after signing)', async function () {
        this.timeout(10000);
        // Set up 2 oracle nodes
        const priv1 = Buffer.alloc(32, 1);
        const priv2 = Buffer.alloc(32, 2);
        const pub1 = secp256k1.publicKeyCreate(priv1, false).slice(1);
        const pub2 = secp256k1.publicKeyCreate(priv2, false).slice(1);
        const node1 = new PublicKey(pub1.slice(0, 32));
        const node2 = new PublicKey(pub2.slice(0, 32));
        await program.methods
            .setOracleNodes([node1, node2, ...Array(6).fill(PublicKey.default)])
            .accounts({
                rampConfig: rampConfigPda,
                authority: admin.publicKey,
            } as any)
            .signers([provider.wallet.payer])
            .rpc();
        // Prepare report_context, message, meta
        const reportContext = Buffer.from('report_context');
        const message = Buffer.from('msg');
        const meta = Buffer.from('meta');
        // Hash = keccak(reportContext || message || meta)
        const Keccak = require('keccak');
        const hasher = new Keccak('keccak256');
        hasher.update(reportContext);
        hasher.update(message);
        hasher.update(meta);
        const hash = hasher.digest();
        // Sign hash with priv1, priv2
        const sigObj1 = secp256k1.ecdsaSign(hash, priv1);
        const sigObj2 = secp256k1.ecdsaSign(hash, priv2);
        const sig1 = Buffer.from(sigObj1.signature);
        const recId1 = sigObj1.recid;
        const sig2 = Buffer.from(sigObj2.signature);
        const recId2 = sigObj2.recid;
        // Tamper message after signing
        const tamperedMessage = Buffer.from('tampered');
        // Assemble data: [rc_len|rc_bytes|msg_len|msg_bytes|meta_len|meta_bytes|sig_count|sig1|rec_id1|sig2|rec_id2]
        const data = Buffer.concat([
            Buffer.from(Uint32Array.of(reportContext.length).buffer),
            reportContext,
            Buffer.from(Uint32Array.of(tamperedMessage.length).buffer),
            tamperedMessage,
            Buffer.from(Uint32Array.of(meta.length).buffer),
            meta,
            Buffer.from(Uint32Array.of(2).buffer), // sig_count
            sig1,
            Buffer.from([recId1]),
            sig2,
            Buffer.from([recId2]),
        ]);
        let threw = false;
        try {
            await program.methods
                .transmit(data as any)
                .accounts({
                    rampConfig: rampConfigPda,
                    authority: admin.publicKey,
                } as any)
                .signers([provider.wallet.payer])
                .rpc();
        } catch (e: any) {
            threw = true;
            if (e.error && e.error.errorName) {
                assert.equal(e.error.errorName, 'Unauthorized', 'Should throw Unauthorized');
            } else if (e.message) {
                assert.include(e.message, 'Unauthorized', 'Should throw Unauthorized');
            } else {
                assert.fail('Error did not contain Unauthorized');
            }
        }
        assert.isTrue(threw, 'Should throw if data is tampered');
    });
});
