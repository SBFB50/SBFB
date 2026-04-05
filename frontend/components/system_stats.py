"""Shared sidebar system stats component."""
import streamlit as st


def render_system_stats():
    """Show CPU, RAM, GPU stats in the sidebar."""
    try:
        import psutil
        cpu = psutil.cpu_percent(interval=0.3)
        ram = psutil.virtual_memory()
        ram_used = ram.used / (1024**3)
        ram_total = ram.total / (1024**3)

        gpu_text = ""
        try:
            import GPUtil
            gpus = GPUtil.getGPUs()
            if gpus:
                g = gpus[0]
                gpu_text = (
                    f"<b>GPU:</b> {g.name} | {g.memoryUsed:.0f}/{g.memoryTotal:.0f} MB "
                    f"({g.memoryUtil*100:.0f}%) | {g.temperature}\u00b0C"
                )
        except Exception:
            pass

        st.sidebar.markdown(
            f'<div style="font-size:11px;color:#888;line-height:1.6">'
            f'<b>CPU:</b> {cpu:.0f}% | '
            f'<b>RAM:</b> {ram_used:.1f}/{ram_total:.0f} GB ({ram.percent:.0f}%)<br>'
            f'{gpu_text}'
            f'</div>',
            unsafe_allow_html=True,
        )
    except Exception:
        pass
