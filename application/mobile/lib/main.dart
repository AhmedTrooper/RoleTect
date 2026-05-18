import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:lucide_icons/lucide_icons.dart';

void main() {
  runApp(const CVSynthMobile());
}

class CVSynthMobile extends StatelessWidget {
  const CVSynthMobile({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'CVSynth Mobile',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        brightness: Brightness.dark,
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF238636),
          brightness: Brightness.dark,
          primary: const Color(0xFF238636),
          surface: const Color(0xFF0D1117),
          surfaceContainer: const Color(0xFF161B22),
        ),
        scaffoldBackgroundColor: const Color(0xFF0D1117),
        textTheme: GoogleFonts.interTextTheme(ThemeData.dark().textTheme),
      ),
      home: const IngestPage(),
    );
  }
}

class IngestPage extends StatefulWidget {
  const IngestPage({super.key});

  @override
  State<IngestPage> createState() => _IngestPageState();
}

class _IngestPageState extends State<IngestPage> {
  final _hostController = TextEditingController();
  final _portController = TextEditingController();
  final _secretController = TextEditingController();
  final _urlController = TextEditingController();
  final _descriptionController = TextEditingController();

  bool _isConnecting = false;
  bool _isSending = false;
  String? _statusMessage;
  Color _statusColor = Colors.white;

  @override
  void initState() {
    super.initState();
    _loadSettings();
  }

  Future<void> _loadSettings() async {
    final prefs = await SharedPreferences.getInstance();
    setState(() {
      _hostController.text = prefs.getString('host') ?? '';
      _portController.text = prefs.getString('port') ?? '14201';
      _secretController.text = prefs.getString('secret') ?? '';
    });
  }

  Future<void> _saveSettings() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('host', _hostController.text.trim());
    await prefs.setString('port', _portController.text.trim());
    await prefs.setString('secret', _secretController.text.trim());
  }

  void _showStatus(String msg, {bool isError = false, bool isSuccess = false}) {
    setState(() {
      _statusMessage = msg;
      if (isError) {
        _statusColor = Colors.redAccent;
      } else if (isSuccess) {
        _statusColor = const Color(0xFF238636);
      } else {
        _statusColor = Colors.blueAccent;
      }
    });
  }

  Future<void> _checkHealth() async {
    if (_hostController.text.isEmpty) {
      _showStatus("Please enter a host IP", isError: true);
      return;
    }

    setState(() => _isConnecting = true);
    final baseUrl = "http://${_hostController.text.trim()}:${_portController.text.trim()}";
    
    try {
      final response = await http.get(Uri.parse("$baseUrl/health")).timeout(const Duration(seconds: 5));
      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        _showStatus("Connected: ${data['message']}", isSuccess: true);
      } else {
        _showStatus("Server error: ${response.statusCode}", isError: true);
      }
    } catch (e) {
      _showStatus("Connection failed: $e", isError: true);
    } finally {
      setState(() => _isConnecting = false);
    }
  }

  Future<void> _sendToVault() async {
    if (_descriptionController.text.trim().isEmpty) {
      _showStatus("Description cannot be empty", isError: true);
      return;
    }

    await _saveSettings();
    setState(() => _isSending = true);

    final baseUrl = "http://${_hostController.text.trim()}:${_portController.text.trim()}";
    final payload = {
      "url": _urlController.text.trim().isEmpty ? null : _urlController.text.trim(),
      "raw_description": _descriptionController.text.trim(),
      "secret": _secretController.text.trim(),
    };

    try {
      final response = await http.post(
        Uri.parse("$baseUrl/ingest"),
        headers: {"Content-Type": "application/json"},
        body: jsonEncode(payload),
      ).timeout(const Duration(seconds: 10));

      if (response.statusCode == 200) {
        _showStatus("Job ingested successfully!", isSuccess: true);
        _descriptionController.clear();
        _urlController.clear();
      } else {
        final errorData = jsonDecode(response.body);
        _showStatus("Error: ${errorData['message'] ?? 'Unknown'}", isError: true);
      }
    } catch (e) {
      _showStatus("Failed to send: $e", isError: true);
    } finally {
      setState(() => _isSending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('CVSynth Vault', style: GoogleFonts.jetBrainsMono(fontWeight: FontWeight.bold)),
        centerTitle: true,
        actions: [
          IconButton(
            icon: _isConnecting 
              ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2))
              : Icon(LucideIcons.wifi),
            onPressed: _checkHealth,
            tooltip: 'Check Connection',
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Status Banner
            if (_statusMessage != null)
              Container(
                padding: const EdgeInsets.all(12),
                margin: const EdgeInsets.only(bottom: 24),
                decoration: BoxDecoration(
                  color: _statusColor.withOpacity(0.1),
                  border: Border.all(color: _statusColor.withOpacity(0.5)),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Row(
                  children: [
                    Icon(_statusColor == Colors.redAccent ? LucideIcons.alertCircle : LucideIcons.info, color: _statusColor, size: 20),
                    const SizedBox(width: 12),
                    Expanded(child: Text(_statusMessage!, style: TextStyle(color: _statusColor, fontWeight: FontWeight.bold, fontSize: 13))),
                    IconButton(
                      icon: Icon(LucideIcons.x, size: 16),
                      onPressed: () => setState(() => _statusMessage = null),
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(),
                    )
                  ],
                ),
              ),

            // Server Config Card
            _buildSectionTitle("SERVER CONFIG"),
            const SizedBox(height: 12),
            Card(
              color: Theme.of(context).colorScheme.surfaceContainer,
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
              child: Padding(
                padding: const EdgeInsets.all(20.0),
                child: Column(
                  children: [
                    _buildTextField(_hostController, "Host IP", "e.g. 192.168.1.15", LucideIcons.globe),
                    const SizedBox(height: 16),
                    Row(
                      children: [
                        Expanded(flex: 2, child: _buildTextField(_portController, "Port", "14201", LucideIcons.hash)),
                        const SizedBox(width: 16),
                        Expanded(flex: 3, child: _buildTextField(_secretController, "Secret Key", "••••••••", LucideIcons.key, isPassword: true)),
                      ],
                    ),
                  ],
                ),
              ),
            ),

            const SizedBox(height: 32),

            // Job Input Card
            _buildSectionTitle("CAPTURE JOB"),
            const SizedBox(height: 12),
            _buildTextField(_urlController, "Source URL (Optional)", "https://...", LucideIcons.externalLink),
            const SizedBox(height: 16),
            _buildTextField(
              _descriptionController, 
              "Job Description", 
              "Paste the JD here...", 
              LucideIcons.fileText,
              maxLines: 8,
            ),

            const SizedBox(height: 32),

            // Action Button
            ElevatedButton.icon(
              onPressed: _isSending ? null : _sendToVault,
              icon: _isSending 
                ? const SizedBox(width: 18, height: 18, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                : Icon(LucideIcons.cpu),
              label: Text(_isSending ? "UPLOADING..." : "INGEST TO VAULT", style: const TextStyle(fontWeight: FontWeight.bold, letterSpacing: 1.2)),
              style: ElevatedButton.styleFrom(
                backgroundColor: Theme.of(context).colorScheme.primary,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(vertical: 20),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
                elevation: 0,
              ),
            ),
            
            const SizedBox(height: 40),
            
            Center(
              child: Opacity(
                opacity: 0.5,
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(LucideIcons.shieldCheck, size: 14),
                    const SizedBox(width: 8),
                    Text("SECURE LOCAL SYNC", style: GoogleFonts.jetBrainsMono(fontSize: 10, fontWeight: FontWeight.bold)),
                  ],
                ),
              ),
            )
          ],
        ),
      ),
    );
  }

  Widget _buildSectionTitle(String title) {
    return Text(
      title,
      style: GoogleFonts.jetBrainsMono(
        fontSize: 11,
        fontWeight: FontWeight.w900,
        color: const Color(0xFF238636),
        letterSpacing: 2,
      ),
    );
  }

  Widget _buildTextField(
    TextEditingController controller, 
    String label, 
    String placeholder, 
    IconData icon, 
    {bool isPassword = false, int maxLines = 1}
  ) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(icon, size: 14, color: Colors.grey),
            const SizedBox(width: 8),
            Text(label, style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: Colors.grey)),
          ],
        ),
        const SizedBox(height: 8),
        TextField(
          controller: controller,
          obscureText: isPassword,
          maxLines: maxLines,
          style: GoogleFonts.inter(fontSize: 14),
          decoration: InputDecoration(
            hintText: placeholder,
            hintStyle: const TextStyle(color: Colors.white24),
            filled: true,
            fillColor: Colors.black.withOpacity(0.2),
            contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide.none),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(12), 
              borderSide: const BorderSide(color: Color(0xFF238636), width: 1)
            ),
          ),
        ),
      ],
    );
  }
}
