#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#define UNICODE
#define _UNICODE

#include <windows.h>

#include <d3d11.h>
#include <dxgi.h>
#include <wrl/client.h>

#include <cstdint>
#include <cwchar>
#include <cstdio>
#include <cstring>
#include <string>

#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "user32.lib")

namespace {

using Microsoft::WRL::ComPtr;

constexpr wchar_t kWindowClassName[] = L"PonteD3D11ProbeWindow";

struct ProbeResult {
    bool ok = false;
    bool hasFeatureLevel = false;
    bool hasAdapter = false;
    D3D_FEATURE_LEVEL featureLevel = static_cast<D3D_FEATURE_LEVEL>(0);
    std::string featureLevelText;
    std::string adapter;
    std::uint32_t vendorId = 0;
    std::uint32_t deviceId = 0;
    std::string hresult;
    std::string stage;
};

LRESULT CALLBACK windowProc(HWND window, UINT message, WPARAM wParam, LPARAM lParam) {
    return DefWindowProcW(window, message, wParam, lParam);
}

HRESULT lastErrorAsHresult() {
    const DWORD error = GetLastError();
    return error == ERROR_SUCCESS ? E_FAIL : HRESULT_FROM_WIN32(error);
}

bool registerWindowClass(HINSTANCE instance, ATOM* atom) {
    WNDCLASSEXW windowClass{};
    windowClass.cbSize = static_cast<UINT>(sizeof(windowClass));
    windowClass.hInstance = instance;
    windowClass.lpfnWndProc = windowProc;
    windowClass.lpszClassName = kWindowClassName;

    *atom = RegisterClassExW(&windowClass);
    return *atom != 0;
}

std::string wideToUtf8(const wchar_t* value) {
    if (value == nullptr || value[0] == L'\0') {
        return {};
    }

    const int required = WideCharToMultiByte(
        CP_UTF8,
        WC_ERR_INVALID_CHARS,
        value,
        -1,
        nullptr,
        0,
        nullptr,
        nullptr);
    if (required <= 1) {
        return {};
    }

    std::string result(static_cast<std::size_t>(required), '\0');
    const int converted = WideCharToMultiByte(
        CP_UTF8,
        WC_ERR_INVALID_CHARS,
        value,
        -1,
        result.data(),
        required,
        nullptr,
        nullptr);
    if (converted <= 1) {
        return {};
    }
    result.resize(static_cast<std::size_t>(converted - 1));
    return result;
}

std::string jsonEscape(const std::string& value) {
    std::string escaped;
    escaped.reserve(value.size() + 8);

    for (const unsigned char character : value) {
        switch (character) {
        case '"':
            escaped += "\\\"";
            break;
        case '\\':
            escaped += "\\\\";
            break;
        case '\b':
            escaped += "\\b";
            break;
        case '\f':
            escaped += "\\f";
            break;
        case '\n':
            escaped += "\\n";
            break;
        case '\r':
            escaped += "\\r";
            break;
        case '\t':
            escaped += "\\t";
            break;
        default:
            if (character < 0x20) {
                char buffer[7]{};
                sprintf_s(buffer, sizeof(buffer), "\\u%04x", static_cast<unsigned int>(character));
                escaped += buffer;
            } else {
                escaped.push_back(static_cast<char>(character));
            }
            break;
        }
    }

    return escaped;
}

std::string formatHresult(HRESULT result) {
    char buffer[11]{};
    sprintf_s(buffer, sizeof(buffer), "0x%08lX", static_cast<unsigned long>(result));
    return buffer;
}

std::string featureLevelToString(D3D_FEATURE_LEVEL featureLevel) {
    switch (featureLevel) {
    case D3D_FEATURE_LEVEL_11_1:
        return "11_1";
    case D3D_FEATURE_LEVEL_11_0:
        return "11_0";
    default: {
        char buffer[7]{};
        sprintf_s(
            buffer,
            sizeof(buffer),
            "0x%04X",
            static_cast<unsigned int>(featureLevel));
        return buffer;
    }
    }
}

void fail(ProbeResult* result, const char* stage, HRESULT hresult) {
    result->ok = false;
    result->stage = stage;
    result->hresult = formatHresult(hresult);
}

void printResult(const ProbeResult& result) {
    const std::string featureLevel = result.hasFeatureLevel
                                         ? "\"" + jsonEscape(result.featureLevelText) + "\""
                                         : "null";
    const std::string adapter = result.hasAdapter ? "\"" + jsonEscape(result.adapter) + "\""
                                                  : "null";
    const std::string vendorId = result.hasAdapter ? std::to_string(result.vendorId) : "null";
    const std::string deviceId = result.hasAdapter ? std::to_string(result.deviceId) : "null";
    const std::string hresult = result.hresult.empty()
                                    ? "null"
                                    : "\"" + jsonEscape(result.hresult) + "\"";
    const std::string stage = result.stage.empty() ? "null" : "\"" + jsonEscape(result.stage) + "\"";

    std::printf(
        "{\"ok\":%s,\"featureLevel\":%s,\"adapter\":%s,\"vendorId\":%s,\"deviceId\":%s,\"hresult\":%s,\"stage\":%s}\n",
        result.ok ? "true" : "false",
        featureLevel.c_str(),
        adapter.c_str(),
        vendorId.c_str(),
        deviceId.c_str(),
        hresult.c_str(),
        stage.c_str());
}

ProbeResult runProbe(bool useWarp) {
    ProbeResult result;
    const HINSTANCE instance = GetModuleHandleW(nullptr);
    ATOM windowClassAtom = 0;
    if (!registerWindowClass(instance, &windowClassAtom)) {
        fail(&result, "register_window_class", lastErrorAsHresult());
        return result;
    }

    HWND window = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        kWindowClassName,
        L"Ponte D3D11 Probe",
        WS_OVERLAPPEDWINDOW,
        0,
        0,
        1,
        1,
        nullptr,
        nullptr,
        instance,
        nullptr);
    if (window == nullptr) {
        const HRESULT hresult = lastErrorAsHresult();
        UnregisterClassW(kWindowClassName, instance);
        fail(&result, "create_window", hresult);
        return result;
    }
    ShowWindow(window, SW_HIDE);

    DXGI_SWAP_CHAIN_DESC swapChainDescription{};
    swapChainDescription.BufferCount = 1;
    swapChainDescription.BufferDesc.Width = 1;
    swapChainDescription.BufferDesc.Height = 1;
    swapChainDescription.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    swapChainDescription.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    swapChainDescription.OutputWindow = window;
    swapChainDescription.SampleDesc.Count = 1;
    swapChainDescription.Windowed = TRUE;
    swapChainDescription.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;

    const D3D_FEATURE_LEVEL requestedFeatureLevels[] = {
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_11_0,
    };
    const D3D_DRIVER_TYPE driverType = useWarp ? D3D_DRIVER_TYPE_WARP : D3D_DRIVER_TYPE_HARDWARE;
    const UINT creationFlags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;

    ComPtr<ID3D11Device> device;
    ComPtr<ID3D11DeviceContext> context;
    ComPtr<IDXGISwapChain> swapChain;
    D3D_FEATURE_LEVEL featureLevel = static_cast<D3D_FEATURE_LEVEL>(0);
    HRESULT hresult = D3D11CreateDeviceAndSwapChain(
        nullptr,
        driverType,
        nullptr,
        creationFlags,
        requestedFeatureLevels,
        static_cast<UINT>(sizeof(requestedFeatureLevels) / sizeof(requestedFeatureLevels[0])),
        D3D11_SDK_VERSION,
        &swapChainDescription,
        &swapChain,
        &device,
        &featureLevel,
        &context);
    if (hresult == E_INVALIDARG) {
        const D3D_FEATURE_LEVEL fallbackFeatureLevels[] = {D3D_FEATURE_LEVEL_11_0};
        swapChain.Reset();
        device.Reset();
        context.Reset();
        featureLevel = static_cast<D3D_FEATURE_LEVEL>(0);
        hresult = D3D11CreateDeviceAndSwapChain(
            nullptr,
            driverType,
            nullptr,
            creationFlags,
            fallbackFeatureLevels,
            static_cast<UINT>(sizeof(fallbackFeatureLevels) / sizeof(fallbackFeatureLevels[0])),
            D3D11_SDK_VERSION,
            &swapChainDescription,
            &swapChain,
            &device,
            &featureLevel,
            &context);
    }
    if (FAILED(hresult)) {
        DestroyWindow(window);
        UnregisterClassW(kWindowClassName, instance);
        fail(&result, "d3d11_create_device_and_swap_chain", hresult);
        return result;
    }

    result.hasFeatureLevel = true;
    result.featureLevel = featureLevel;
    result.featureLevelText = featureLevelToString(featureLevel);

    ComPtr<IDXGIDevice> dxgiDevice;
    hresult = device.As(&dxgiDevice);
    if (FAILED(hresult)) {
        fail(&result, "query_dxgi_device", hresult);
    } else {
        ComPtr<IDXGIAdapter> adapter;
        hresult = dxgiDevice->GetAdapter(&adapter);
        if (FAILED(hresult)) {
            fail(&result, "get_dxgi_adapter", hresult);
        } else {
            DXGI_ADAPTER_DESC adapterDescription{};
            hresult = adapter->GetDesc(&adapterDescription);
            if (FAILED(hresult)) {
                fail(&result, "get_adapter_description", hresult);
            } else {
                result.adapter = wideToUtf8(adapterDescription.Description);
                if (result.adapter.empty() && adapterDescription.Description[0] != L'\0') {
                    fail(
                        &result,
                        "convert_adapter_description",
                        HRESULT_FROM_WIN32(ERROR_NO_UNICODE_TRANSLATION));
                } else {
                    result.hasAdapter = true;
                    result.vendorId = adapterDescription.VendorId;
                    result.deviceId = adapterDescription.DeviceId;
                }
            }
        }
    }

    if (!result.stage.empty()) {
        DestroyWindow(window);
        UnregisterClassW(kWindowClassName, instance);
        return result;
    }

    ComPtr<ID3D11Texture2D> backBuffer;
    hresult = swapChain->GetBuffer(0, IID_PPV_ARGS(&backBuffer));
    if (FAILED(hresult)) {
        DestroyWindow(window);
        UnregisterClassW(kWindowClassName, instance);
        fail(&result, "get_back_buffer", hresult);
        return result;
    }

    ComPtr<ID3D11RenderTargetView> renderTarget;
    hresult = device->CreateRenderTargetView(backBuffer.Get(), nullptr, &renderTarget);
    if (FAILED(hresult)) {
        DestroyWindow(window);
        UnregisterClassW(kWindowClassName, instance);
        fail(&result, "create_render_target_view", hresult);
        return result;
    }

    const FLOAT clearColor[] = {0.02F, 0.08F, 0.18F, 1.0F};
    context->OMSetRenderTargets(1, renderTarget.GetAddressOf(), nullptr);
    context->ClearRenderTargetView(renderTarget.Get(), clearColor);

    hresult = swapChain->Present(0, 0);
    if (FAILED(hresult)) {
        DestroyWindow(window);
        UnregisterClassW(kWindowClassName, instance);
        fail(&result, "present", hresult);
        return result;
    }

    result.ok = true;
    DestroyWindow(window);
    UnregisterClassW(kWindowClassName, instance);
    return result;
}

} // namespace

int wmain(int argc, wchar_t** argv) {
    bool useWarp = false;
    for (int index = 1; index < argc; ++index) {
        if (std::wcscmp(argv[index], L"--warp") == 0) {
            useWarp = true;
            continue;
        }

        ProbeResult result;
        fail(&result, "arguments", E_INVALIDARG);
        printResult(result);
        return 2;
    }

    const ProbeResult result = runProbe(useWarp);
    printResult(result);
    return result.ok ? 0 : 1;
}
