import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubOpenSourceComponent } from './hub-open-source.component';

describe('HubOpenSourceComponent', () => {
  let component: HubOpenSourceComponent;
  let fixture: ComponentFixture<HubOpenSourceComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubOpenSourceComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubOpenSourceComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
